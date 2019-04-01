mod ambient_lighting_system;
mod directional_lighting_system;
mod frame;
pub mod model;
pub mod pick;
mod point_lighting_system;
mod pp_system;
mod scene_system;
mod shadow;
mod skybox;
pub mod texture;
mod ui;

use pick::Object3DPicker;
use ui::GuiRenderer;

use imgui::{ImGui, Ui};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};

use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

use std::sync::Arc;

use crate::camera::Camera;
use crate::ecs::components::{LightComponent, LightType, ModelComponent, TransformComponent};
use crate::ecs::{Entity, ECS};
use crate::error::{TwError, TwResult};
use crate::event::Event;
use crate::resource::Resources;
use frame::{FrameSystem, Pass};
use scene_system::SceneDrawSystem;

use vulkano::image::AttachmentImage;
use vulkano::sampler::Sampler;

pub struct GBufferComponent {
    pub image: Arc<AttachmentImage>,
    pub sampler: Arc<Sampler>,
}

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

    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<GpuFuture>>,

    frame_system: FrameSystem,

    // Special pipelines and stuff for the UI
    pub gui: GuiRenderer,
    // pipeline for mouse picking
    pub object_picker: Object3DPicker,
    scene_system: SceneDrawSystem,
}

impl<'a> Renderer<'a> {
    pub fn new(
        imgui: &mut ImGui,
        instance: &'a Arc<Instance>,
        surface: Arc<Surface<winit::Window>>,
    ) -> TwResult<Self> {
        let window = surface.window();
        // List of device.
        // In real life -> Maybe all the device cannot draw. Should leave the device
        // choice to the user... Here we get the first one that comes
        let physical = PhysicalDevice::enumerate(&instance).next().ok_or(
            TwError::RenderingSystemInitialization("Cannot get physical device".to_owned()),
        )?;

        // Get the queue to write command to the GPU. queues should support graphics
        // and should be able to write to our surface
        let queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
            .ok_or(TwError::RenderingSystemInitialization(
                "Cannot find graphic queue family".to_owned(),
            ))?;

        // Now we can initialize the vulkan device. Needs five parameters
        // - which physical device to connect to
        // - a list of optional features and extensions that our program needs to work
        // correctly on. Here we only need khr_swapchain extension that allows us to draw
        // to a window
        // - List of layers to enable (?) very niche so use None in example.
        // - list of queues that we are going to use. Exact parameters is a iteraotr
        // which items are (Queue, f32) where float is priority between 0 and 1. (hint)
        let device_ext = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &device_ext,
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("Could not create device");

        let queue = queues.next().ok_or(TwError::RenderingSystemInitialization(
            "Cannot find queue".to_owned(),
        ))?;

        // Create the swapchain. Creating a swapchain allocates the color buffers that
        // will contain the image that will be visible on screen.
        let (swapchain, images) = {
            let caps = surface.capabilities(physical)?;
            let usage = caps.supported_usage_flags;
            // alpha mode indicates how the alpha value of the final image will behave.
            let alpha = caps.supported_composite_alpha.iter().next().ok_or(
                TwError::RenderingSystemInitialization(
                    "Cannot find supported composite alpha when creating swapchain".to_owned(),
                ),
            )?;
            let format = caps.supported_formats[0].0;
            println!("{:?}", caps);
            let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
                // convert to physical pixels
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                // The window no longer exists so exit the application.
                panic!("BOUM");
            };

            // Please take a look at the docs for the meaning of the parameters we didn't mention.

            let tmp = Swapchain::new(
                device.clone(),
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
                None,
            )?;
            tmp
        };

        let dimensions = images[0].dimensions();
        let frame_system = FrameSystem::new(queue.clone(), swapchain.format());

        // Initialization is finally finished!
        let recreate_swapchain = false;

        let scene_system = timed!(SceneDrawSystem::new(
            queue.clone(),
            frame_system.deferred_subpass(),
            dimensions
        ));

        let (gui, gui_fut) = timed!(GuiRenderer::new(
            imgui,
            surface.clone(),
            frame_system.ui_subpass(),
            queue.clone()
        ));

        let object_picker = timed!(Object3DPicker::new(
            device.clone(),
            queue.clone(),
            surface.clone(),
            dimensions
        ));

        let previous_frame_end = Some(gui_fut);
        Ok(Renderer {
            surface,
            _physical: physical,
            device,
            queue,
            swapchain,
            images,

            recreate_swapchain,
            previous_frame_end,

            frame_system,
            gui,
            object_picker,
            dimensions,
            scene_system,
        })
    }

    /// This is to update camera projection matrix
    pub fn dimensions(&self) -> [u32; 2] {
        self.dimensions
    }

    // To be called at every main loop iteration.
    pub fn render<'ui>(
        &mut self,
        resources: &Resources,
        ui: Ui<'ui>,
        camera: &mut Camera,
        lights: Vec<(&LightComponent, &TransformComponent)>,
        objects: Vec<(&ModelComponent, &TransformComponent)>,
    ) {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
        let window = self.surface.window();

        if self.recreate_swapchain {
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) =
                match self.swapchain.recreate_with_dimension(dimensions) {
                    Ok(r) => r,
                    // This error tends to happen when the user is manually resizing the window.
                    // Simply restarting the loop is the easiest way to fix this issue.
                    Err(SwapchainCreationError::UnsupportedDimensions) => return,
                    Err(err) => panic!("{:?}", err),
                };

            self.swapchain = new_swapchain;
            self.images = new_images;
            let dimensions = self.images[0].dimensions();

            self.gui.rebuild_pipeline(self.frame_system.ui_subpass());
            self.scene_system
                .rebuild_pipeline(self.frame_system.deferred_subpass(), dimensions);
            self.object_picker.rebuild_pipeline(dimensions);

            // hey there
            camera.set_aspect((dimensions[0] as f32) / (dimensions[1] as f32));

            self.dimensions = dimensions;
            self.recreate_swapchain = false;
        }

        // Before we can draw on the output, we have to *acquire* an image from the
        // swapchain. If no image is available, (which happens if you submit draw
        // commands too quickly), then the function will block.
        // This operation returns the index of the image that we are allowed to draw upon.
        // None can be a timeout instead.
        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(err) => panic!("{:?}", err),
            };

        let reference = self
            .previous_frame_end
            .take()
            .expect("There should be a Future in there");
        let future = reference.join(acquire_future);

        let mut frame = self
            .frame_system
            .frame(future, self.images[image_num].clone());
        let mut after_future = None;
        let cb = Arc::new(self.scene_system.draw(resources, camera, &objects));
        let gui_cb = Arc::new(self.gui.render(ui));

        while let Some(pass) = frame.next_pass() {
            match pass {
                Pass::Shadow(mut shadow_pass) => {
                    // Draw scene with directional light.
                    for (light, transform) in lights.iter() {
                        // only one light that cast shadow.
                        if light.cast_shadows {
                            shadow_pass.draw_shadow_map(resources, transform, &objects);
                            break;
                        }
                    }
                }
                Pass::Deferred(mut draw_pass) => {
                    draw_pass.execute(cb.clone());
                }
                Pass::Lighting(mut lighting_pass) => {
                    //                    lighting_pass.ambient_light([0.5, 0.5, 0.5]);
                    let max_nb_cast = 1;
                    let mut total_cast_shadows = 0;
                    for (light, transform) in lights.iter() {
                        let mut should_cast_shadows = false;
                        if light.cast_shadows && total_cast_shadows < max_nb_cast {
                            should_cast_shadows = true;
                            total_cast_shadows += 1;
                        }

                        match &light.light_type {
                            LightType::Directional => {
                                if should_cast_shadows {
                                    lighting_pass.directional_light_with_shadows(
                                        transform.position,
                                        light.color,
                                    );
                                } else {
                                    lighting_pass
                                        .directional_light(transform.position, light.color);
                                }
                            }
                            LightType::Point => {
                                lighting_pass.point_light(transform.position, light.color);
                            }
                            LightType::Ambient => {
                                lighting_pass.ambient_light(light.color);
                            }
                        }
                    }
                }
                Pass::Skybox(mut sky_pass) => {
                    sky_pass.draw_skybox(camera);
                }
                Pass::PostProcessing(mut post_processing) => {
                    post_processing.outlines();
                }
                Pass::Gui(mut draw_pass) => {
                    draw_pass.execute(gui_cb.clone());
                }
                Pass::Finished(af) => {
                    after_future = Some(af);
                }
            }
        }

        let future = after_future
            .unwrap()
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
    pub fn pick_object(
        &mut self,
        x: f64,
        y: f64,
        ecs: &ECS,
        resources: &Resources,
    ) -> Option<Entity> {
        self.object_picker.pick_object(x, y, ecs, &resources.models)
    }

    pub fn handle_events(&mut self, events: &Vec<Event>) {
        for ev in events {
            self.scene_system.handle_event(&ev);
            self.frame_system.handle_event(&ev);
        }
    }
}
