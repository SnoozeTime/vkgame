pub mod model;
pub mod texture;
pub mod pick;
mod scene_system;
mod ui;

use ui::GuiRenderer;
use pick::Object3DPicker;

use imgui::{ImGui, Ui};
use vulkano::image::attachment::AttachmentImage;
use vulkano::format::Format;
use vulkano::command_buffer::{AutoCommandBufferBuilder};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::swapchain::{Surface, AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use winit::Window;
use std::sync::Arc;

use crate::error::{TwError, TwResult};
use crate::resource::Resources;
use crate::camera::Camera;
use crate::ecs::components::{TransformComponent, ModelComponent, LightComponent};
use crate::ecs::{Entity, ECS};
use scene_system::SceneDrawSystem;


pub struct Renderer<'a> {
    pub surface: Arc<Surface<winit::Window>>,
    _physical: PhysicalDevice<'a>,
    dimensions: [u32; 2],

    // logical device.
    pub device: Arc<Device>,
    // command queue for our system. Supports graphics for the window.
    pub queue: Arc<Queue>,

    // Swapchain stuff
    swapchain: Arc<Swapchain<winit::Window>>,
    images: Vec<Arc<SwapchainImage<winit::Window>>>,

    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<GpuFuture>>,

    // Special pipelines and stuff for the UI
    pub gui: GuiRenderer,
    // pipeline for mouse picking
    pub object_picker: Object3DPicker,
    scene_system: SceneDrawSystem,
}

impl<'a> Renderer<'a> {

    pub fn new(imgui: &mut ImGui, instance: &'a Arc<Instance>, surface: Arc<Surface<winit::Window>>) -> TwResult<Self> {

        let window = surface.window();
        // List of device.
        // In real life -> Maybe all the device cannot draw. Should leave the device
        // choice to the user... Here we get the first one that comes
        let physical = PhysicalDevice::enumerate(&instance)
            .next()
            .ok_or(TwError::RenderingSystemInitialization("Cannot get physical device".to_owned()))?;

        // Get the queue to write command to the GPU. queues should support graphics
        // and should be able to write to our surface
        let queue_family = physical.queue_families()
            .find(|&q| {
                q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
            })
        .ok_or(TwError::RenderingSystemInitialization("Cannot find graphic queue family".to_owned()))?;

        // Now we can initialize the vulkan device. Needs five parameters
        // - which physical device to connect to
        // - a list of optional features and extensions that our program needs to work
        // correctly on. Here we only need khr_swapchain extension that allows us to draw
        // to a window
        // - List of layers to enable (?) very niche so use None in example.
        // - list of queues that we are going to use. Exact parameters is a iteraotr
        // which items are (Queue, f32) where float is priority between 0 and 1. (hint)
        let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
        let (device, mut queues) = Device::new(physical,
                                               physical.supported_features(),
                                               &device_ext,
                                               [(queue_family, 0.5)].iter().cloned()).expect("Could not create device");

        let queue = queues.next()
            .ok_or(TwError::RenderingSystemInitialization("Cannot find queue".to_owned()))?;

        // Create the swapchain. Creating a swapchain allocates the color buffers that
        // will contain the image that will be visible on screen.
        let (swapchain, images) = {

            let caps = surface.capabilities(physical)?;
            println!("{:?}", caps);
            let usage = caps.supported_usage_flags;
            // alpha mode indicates how the alpha value of the final image will behave.
            let alpha = caps.supported_composite_alpha.iter().next()
                .ok_or(TwError::RenderingSystemInitialization("Cannot find supported composite alpha when creating swapchain".to_owned()))?;
            let format = caps.supported_formats[0].0;
            let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
                // convert to physical pixels
                let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                // The window no longer exists so exit the application.
                panic!("BOUM");
            };

            // Please take a look at the docs for the meaning of the parameters we didn't mention.
            Swapchain::new(device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            initial_dimensions,
            1,
            usage,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None)?

        };


        // at this point opengl init would be finished but vulkna requires more.
        // Render pass is an object that describes where the output of the graphic
        // pipeline will go. It describes the layout of the images where the color
        // depth and/or stencil info will be written.
        // TODO new render pass for picking.
        let render_pass = Arc::new(vulkano::ordered_passes_renderpass!(
                device.clone(),
                attachments: {
                    // `color` is a custom name we give to the first and only attachment.
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                passes: [
                {
                    color: [color],
                    depth_stencil: {depth},
                    input: []
                },

                // Pass for the GUI
                {
                    color: [color],
                    depth_stencil: {},
                    input: []
                }
                ]
        ).unwrap());

        // The render pass we created above only describes the layout of our framebuffers. Before we
        // can draw we also need to create the actual framebuffers.
        // Since we need to draw to multiple images, we are going to create a different framebuffer for
        // each image.
        let (framebuffers, dimensions) = window_size_dependent_setup(device.clone(), &images, render_pass.clone());
        println!("After framebuffer, dimensions: {:?}", dimensions);
        // Initialization is finally finished!
        let recreate_swapchain = false;
        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<GpuFuture>);

        let scene_system = SceneDrawSystem::new(queue.clone(), Subpass::from(render_pass.clone(), 0).unwrap(), dimensions);
        let gui = GuiRenderer::new(imgui, surface.clone(), Subpass::from(render_pass.clone(), 1).unwrap(), queue.clone());
        let object_picker = Object3DPicker::new(device.clone(),
                                                queue.clone(),
                                                surface.clone(),
                                                dimensions);
        Ok(Renderer {
            surface,
            _physical: physical,
            device, 
            queue,
            swapchain,
            images,

            render_pass,
            framebuffers,
            recreate_swapchain,
            previous_frame_end,
            gui,
            object_picker,
            dimensions,
            scene_system,
        })
    }

    /// This is to update camera projection matrix
    pub fn dimensions(&self) -> [u32;2] { self.dimensions }

    // To be called at every main loop iteration.
    pub fn render<'ui>(&mut self,
                       resources: &Resources,
                       ui: Ui<'ui>,
                       camera: &mut Camera,
                       lights: Vec<(&LightComponent, &TransformComponent)>,
                       objects: Vec<(&ModelComponent, &TransformComponent)>) {

        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
        let window = self.surface.window();

        if self.recreate_swapchain {
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(err) => panic!("{:?}", err)
            };

            self.swapchain = new_swapchain;
            self.images = new_images;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            let (new_framebuffers, dimensions) = window_size_dependent_setup(self.device.clone(),
            &self.images, 
            self.render_pass.clone());

            self.gui.rebuild_pipeline(
                Subpass::from(self.render_pass.clone(), 1).unwrap(),
                );
            self.scene_system.rebuild_pipeline(Subpass::from(self.render_pass.clone(), 0).unwrap(), dimensions);
            self.object_picker.rebuild_pipeline(dimensions);

            // hey there
            camera.set_aspect((dimensions[0] as f32) / (dimensions[1] as f32));

            self.dimensions = dimensions;
            self.framebuffers = new_framebuffers;
            self.recreate_swapchain = false;
        }

        // Before we can draw on the output, we have to *acquire* an image from the
        // swapchain. If no image is available, (which happens if you submit draw
        // commands too quickly), then the function will block.
        // This operation returns the index of the image that we are allowed to draw upon.
        // None can be a timeout instead.
        let (image_num, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.recreate_swapchain = true;
                return;
            },
            Err(err) => panic!("{:?}", err)
        };

        // Specify the color to clear the framebuffer with.
        let clear_values = vec!([0.0, 0.0, 0.0, 1.0].into(), 1f32.into());

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family()).unwrap()
            .begin_render_pass(self.framebuffers[image_num].clone(), false, clear_values)
            .unwrap();
        unsafe {
            command_buffer_builder = command_buffer_builder.execute_commands(
                self.scene_system.draw(resources, camera, lights, objects)
                ).unwrap();
        }

        // Second subpass.
        command_buffer_builder = command_buffer_builder.next_subpass(false).unwrap();
        // Now display the GUI.
        command_buffer_builder = self.gui.render(command_buffer_builder, ui); 

        // Finish render pass
        command_buffer_builder = command_buffer_builder.end_render_pass()
            .unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = command_buffer_builder.build().unwrap();


        let reference = self.previous_frame_end.take().expect("There should be a Future in there");
        let future = reference.join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer).unwrap()
            // The color output is now expected to contain our triangle. But in order to show it on
            // the screen, we have to *present* the image by calling `present`.
            //
            // This function does not actually present the image immediately. Instead it submits a
            // present command at the end of the queue. This means that it will only be presented once
            // the GPU has finished executing the command buffer that draws the triangle.
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())) as Box<_>);
            }
        }

    }

    /// Picking an object works by storing Entity ID in color attachment then
    /// finding what pixel has been clicked by the mouse.
    pub fn pick_object(&mut self, x: f64, y: f64, ecs: &ECS, resources: &Resources) -> Option<Entity> {
        self.object_picker.pick_object(x, y, ecs, &resources.models)
    }
}


/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    device: Arc<Device>,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    ) -> (Vec<Arc<FramebufferAbstract + Send + Sync>>, [u32; 2]) {
    let dimensions = images[0].dimensions();

    let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();
    let framebuffers = images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
            .add(image.clone()).unwrap()
            .add(depth_buffer.clone()).unwrap()
            .build().unwrap()
        ) as Arc<FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>();

    (framebuffers, dimensions)
}   

