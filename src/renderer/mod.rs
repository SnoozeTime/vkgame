pub mod model;
pub mod texture;
mod pick;
mod ui;

use ui::GuiRenderer;
use pick::Object3DPicker;

use image::ImageBuffer;
use image::Rgba;
use imgui::{ImGui, Ui};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use vulkano::image::attachment::AttachmentImage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::buffer::BufferUsage;
use vulkano::format::Format;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::command_buffer::{DynamicState, CommandBuffer, AutoCommandBufferBuilder};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{Surface, AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use winit::Window;
use std::sync::Arc;
use std::iter;
use std::io::Write;
use std::fs::File;
use cgmath::Matrix4;

use crate::error::{TwError, TwResult};
use crate::resource::Resources;
use crate::camera::Camera;
use crate::ecs::components::{TransformComponent, ModelComponent, LightComponent};
use crate::ecs::{Entity, ECS, gen_index::GenerationalIndex};
use self::model::{Vertex, ModelManager};
use self::texture::TextureManager;

// Can have multiple pipelines in an application. In
// particular, you need a pipeline for each combinaison
// of shaders.
pub struct PipelineState {
    pub pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vs: vs::Shader,
    fs: fs::Shader,
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

    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pub pipeline: PipelineState,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,

    pub uniform_buffer: CpuBufferPool<vs::ty::Data>,
    pub light_buffer: CpuBufferPool<fs::ty::Data>,

    pub recreate_swapchain: bool,
    pub previous_frame_end: Option<Box<GpuFuture>>,

    // RESOURCES
    pub texture_manager: TextureManager,
    pub model_manager: ModelManager,

    // Special pipelines and stuff for the UI
    pub gui: GuiRenderer,

    // pipeline for mouse picking
    pub object_picker: Object3DPicker,
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
        let render_pass = Arc::new(vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    // `color` is a custom name we give to the first and only attachment.
                    color: {
                        // `load: Clear` means that we ask the GPU to clear the content of this
                        // attachment at the start of the drawing.
                        load: Clear,
                        // `store: Store` means that we ask the GPU to store the output of the draw
                        // in the actual image. We could also ask it to discard the result.
                        store: Store,
                        // `format: <ty>` indicates the type of the format of the image. This has to
                        // be one of the types of the `vulkano::format` module (or alternatively one
                        // of your structs that implements the `FormatDesc` trait). Here we use the
                        // same format as the swapchain.
                        format: swapchain.format(),
                        // TODO:
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: Format::D16Unorm,
                        samples: 1,
                    }
                },
                pass: {
                    // We use the attachment named `color` as the one and only color attachment.
                    color: [color],
                    // No depth-stencil attachment is indicated with empty brackets.
                    depth_stencil: {depth}
                }
        ).unwrap());


        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();

        // The render pass we created above only describes the layout of our framebuffers. Before we
        // can draw we also need to create the actual framebuffers.
        // Since we need to draw to multiple images, we are going to create a different framebuffer for
        // each image.
        let (pipeline, framebuffers, dimensions) = window_size_dependent_setup(device.clone(), &vs, &fs, &images, render_pass.clone());
        println!("After framebuffer, dimensions: {:?}", dimensions);
        // Initialization is finally finished!
        let recreate_swapchain = false;
        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<GpuFuture>);

        let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());
        let light_buffer = CpuBufferPool::<fs::ty::Data>::new(device.clone(), BufferUsage::all());

        let gui = GuiRenderer::new(imgui, surface.clone(),
        device.clone(),
        render_pass.clone(),
        queue.clone());
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
            pipeline: PipelineState { pipeline, vs, fs},
            framebuffers,
            uniform_buffer,
            light_buffer,
            recreate_swapchain,
            previous_frame_end,
            texture_manager: TextureManager::new(),
            model_manager: ModelManager::new(),
            gui,
            object_picker,
            dimensions
        })
    }

    /// This is to update camera projection matrix
    pub fn dimensions(&self) -> [u32;2] { self.dimensions }

    /*
     * Store a new texture in the texture manager
     * */
    pub fn load_texture(&mut self,
                        texture_name: String,
                        texture_path: &std::path::Path,
                        width: u32,
                        height: u32) -> TwResult<()> {

        self.texture_manager.load_texture(
            texture_name,
            texture_path,
            width,
            height,
            self.device.clone(),
            self.queue.clone())?;
        Ok(())
    }

    /*
     * Store a new model in the model manager
     * */
    pub fn load_model(&mut self,
                      model_name: String,
                      model_path: &std::path::Path) -> TwResult<()> {
        self.model_manager.load_model(model_name, model_path, self.device.clone())?;
        Ok(())
    }


    // To be called at every main loop iteration.
    pub fn render<'ui>(&mut self,
                       resources: &Resources,
                       ui: Ui<'ui>,
                       camera: &mut Camera,
                       lights: Vec<(&LightComponent, &TransformComponent)>,
                       objects: Vec<(&ModelComponent, &TransformComponent)>) {

        let (view, proj) = camera.get_vp(); 
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
            let (new_pipeline, new_framebuffers, dimensions) = window_size_dependent_setup(self.device.clone(),
            &self.pipeline.vs, 
            &self.pipeline.fs, 
            &self.images, 
            self.render_pass.clone());

            self.gui.rebuild_pipeline(self.device.clone(),
            self.render_pass.clone());
            self.object_picker.rebuild_pipeline(dimensions);

            // hey there
            camera.set_aspect((dimensions[0] as f32) / (dimensions[1] as f32));

            self.dimensions = dimensions;
            self.pipeline.pipeline = new_pipeline;
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

        // 1st thing: Get the lighting data
        // ---------------------------------------------
        let (color, position) = if lights.len() > 0 {
            let (light, transform) = lights[0];
            (light.color, transform.position.into())
        } else {
            ([0.5, 0.5, 0.5], [5.0, 0.5, 1.0])
        };

        let light_buffer = {
            let data = fs::ty::Data {
                color,
                position,
                _dummy0: [0;4], // wtf is that?
            };
            self.light_buffer.next(data).unwrap()
        };


        // 2nd thing: Draw all objects
        // ------------------------------
        for (model, transform) in objects.iter() {
            let texture = resources.textures.textures.get(
                &model.texture_name
            ).unwrap();


            // BUILD DESCRIPTOR SETS.
            // 1. For texture
            let tex_set = Arc::new(
                PersistentDescriptorSet::start(self.pipeline.pipeline.clone(), 1)
                .add_sampled_image(texture.texture.clone(), texture.sampler.clone()).unwrap()
                .add_buffer(light_buffer.clone()).unwrap()
                .build().unwrap()
            );


            let model = resources.models.models.get(
                &model.mesh_name
            ).unwrap();

            let uniform_buffer_subbuffer = {
                let uniform_data = create_mvp(transform, &view, &proj);
                self.uniform_buffer.next(uniform_data).unwrap()
            };

            let set = Arc::new(PersistentDescriptorSet::start(self.pipeline.pipeline.clone(), 0)
                               .add_buffer(uniform_buffer_subbuffer).unwrap()
                               .build().unwrap()
            );

            command_buffer_builder = command_buffer_builder
                .draw_indexed(self.pipeline.pipeline.clone(),
                &DynamicState::none(),
                vec![model.vertex_buffer.clone()],
                model.index_buffer.clone(),
                (set.clone(), tex_set.clone()),
                ()).unwrap();
        }

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

    fn get_pos(&self, x: usize, y: usize) -> usize {
        4 * (y * self.dimensions[0] as usize + x)
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
    vs: &vs::Shader,
    fs: &fs::Shader,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    ) -> (Arc<GraphicsPipelineAbstract + Send + Sync>, Vec<Arc<FramebufferAbstract + Send + Sync>>, [u32; 2]) {
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

    let pipeline = Arc::new(GraphicsPipeline::start()
                            .vertex_input_single_buffer::<Vertex>()
                            .vertex_shader(vs.main_entry_point(), ())
                            .triangle_list()
                            //.cull_mode_back()
                            .viewports_dynamic_scissors_irrelevant(1)
                            .depth_stencil_simple_depth()
                            .viewports(iter::once(Viewport {
                                origin: [0.0, 0.0],
                                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                depth_range: 0.0 .. 1.0,
                            }))
                            .fragment_shader(fs.main_entry_point(), ())
                            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                            .build(device.clone())
                            .unwrap());

    (pipeline, framebuffers, dimensions)
}   

pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/main.vert"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/main.frag"
    }
}

pub fn create_mvp(t: &TransformComponent, view: &Matrix4<f32>, proj: &Matrix4<f32>) -> vs::ty::Data {
    let scale = t.scale;
    let model = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
        * Matrix4::from_translation(t.position);


    vs::ty::Data {
        model: model.into(),
        view: (*view).into(),
        proj: (*proj).into(),
    }
}


