use vulkano::pipeline::viewport::Viewport;
use vulkano::device::{Device, Queue};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState};
use vulkano::swapchain::Surface;
use vulkano::image::ImageUsage;
use vulkano::format::Format;
use vulkano::sync::GpuFuture;
use vulkano::image::attachment::AttachmentImage;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::buffer::cpu_pool::CpuBufferPool;
use std::sync::Arc;
use std::iter;

use crate::ecs::{Entity, ECS, gen_index::GenerationalIndex};
use super::model::{ModelManager, Vertex};
use super::scene_system::{create_mvp, vs};

/*
 * This module will implement a technique to get the entity when clicking on a
 * pixel of the screen. On click, it will render the game objects to a color
 * attachment.
 * Instead of the real color, the color data will be the entity ID.
 *
 * Then, by using the position of the mouse and reading from the created image,
 * we can get the entity ID.
 * */

pub struct PickPipelineState {
    pub pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vs: pick_vs::Shader,
    fs: pick_fs::Shader,
}

impl PickPipelineState {

    pub fn new(device: Arc<Device>,
               render_pass: Arc<RenderPassAbstract + Send + Sync>,
               dimensions: [u32; 2]) -> Self {
        let vs = pick_vs::Shader::load(device.clone()).unwrap();
        let fs = pick_fs::Shader::load(device.clone()).unwrap();
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

        PickPipelineState {
            fs,
            vs,
            pipeline,
        }
    }

    /// Not ideal, but this needs to be called when we change the screen dimensions.
    pub fn rebuild_pipeline(&mut self,
                            device: Arc<Device>,
                            render_pass: Arc<RenderPassAbstract + Send + Sync>,
                            dimensions: [u32; 2]) {

        self.pipeline = Arc::new(GraphicsPipeline::start()
                                 .vertex_input_single_buffer::<Vertex>()
                                 .vertex_shader(self.vs.main_entry_point(), ())
                                 .triangle_list()
                                 //.cull_mode_back()
                                 .viewports_dynamic_scissors_irrelevant(1)
                                 .depth_stencil_simple_depth()
                                 .viewports(iter::once(Viewport {
                                     origin: [0.0, 0.0],
                                     dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                                     depth_range: 0.0 .. 1.0,
                                 }))
                                 .fragment_shader(self.fs.main_entry_point(), ())
                                 .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                                 .build(device.clone())
                                 .unwrap())
    }
}


pub mod pick_vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/pick.vert"
    }
}

mod pick_fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/pick.frag"
    }
}


pub struct Object3DPicker {
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Arc<Surface<winit::Window>>,
    dimensions: [u32; 2],

    // Does not use same image format as the normal render pass so need to create 
    // a new one.
    render_pass: Arc<RenderPassAbstract + Send + Sync>,

    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    pub pipeline: PickPipelineState,
    pub framebuffer: Arc<FramebufferAbstract + Send + Sync>,
    pub image: Arc<AttachmentImage>,
    // Async instantiation. First click will be slow :D
    pub buf: Option<Arc<CpuAccessibleBuffer<[u8]>>>,
}


impl Object3DPicker {

    pub fn new(device: Arc<Device>,
               queue: Arc<Queue>,
               surface: Arc<Surface<winit::Window>>,
               dimensions: [u32; 2]) -> Self {

        let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());
        let usage = ImageUsage {
            transfer_source: true,
            .. ImageUsage::none()
        };
        let image = AttachmentImage::with_usage(
            device.clone(), 
            dimensions,
            Format::R8G8B8A8Unorm, //B8G8R8A8Srgb,
            usage).unwrap();
        let depth_buffer = AttachmentImage::transient(device.clone(),
        dimensions,
        Format::D16Unorm).unwrap();

        // Create the render pass. It will use images that have format rgbunorm
        let render_pass = Arc::new(vulkano::single_pass_renderpass!(
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format:Format::R8G8B8A8Unorm,
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
                    color: [color],
                    depth_stencil: {depth}
                }
        ).unwrap());

        let framebuffer = Arc::new(Framebuffer::start(render_pass.clone())
                                   .add(image.clone()).unwrap()
                                   .add(depth_buffer.clone()).unwrap()
                                   .build().unwrap());


        let pipeline = PickPipelineState::new(
            device.clone(),
            render_pass.clone(),
            dimensions);

        Object3DPicker {
            device: device.clone(),
            queue: queue.clone(),
            surface: surface.clone(),
            dimensions,

            render_pass,
            pipeline,
            framebuffer,
            uniform_buffer,
            image,
            buf: None,
        }
    }

    fn create_buffer(&mut self, dimensions: [u32; 2]) {
        // This will contain the data copied from CPU.
        self.buf = timed!(Some(CpuAccessibleBuffer::from_iter(
                    self.queue.device().clone(),
                    BufferUsage::all(),
                    (0 .. dimensions[0] * dimensions[1] * 4).map(|_| 0u8))
                .expect("failed to create buffer")));
    }

    pub fn create_pushconstants(col: usize) -> pick_vs::ty::PushConstants {

        pick_vs::ty::PushConstants {
            color: 
                [
                ((col & 0xFF) as f32) / 255.0,
                ((col >> 8) & 0xFF) as f32 / 255.0,
                ((col >> 16) & 0xFF) as f32 / 255.0,
                1.0], // Transparent means no entity.
        }
    }

    pub fn get_entity_id(r: u8, g: u8, b: u8, a: u8) -> Option<usize> {
        if a == 0 {
            None
        } else {
            Some((r as usize) | (g as usize) << 8 | (b as usize) << 16)
        }
    }

    pub fn rebuild_pipeline(&mut self,
                            dimensions: [u32; 2]) {

        self.dimensions = dimensions;

        let usage = ImageUsage {
            transfer_source: true,
            .. ImageUsage::none()
        };
        self.image = AttachmentImage::with_usage(
            self.device.clone(), 
            dimensions,
            Format::R8G8B8A8Unorm,
            usage).unwrap();
        let depth_buffer = AttachmentImage::transient(self.device.clone(),
        dimensions,
        Format::D16Unorm).unwrap();
        self.buf = None;
        self.framebuffer = Arc::new(Framebuffer::start(self.render_pass.clone())
                                    .add(self.image.clone()).unwrap()
                                    .add(depth_buffer.clone()).unwrap()
                                    .build().unwrap());

        self.pipeline.rebuild_pipeline(self.device.clone(),
        self.render_pass.clone(), dimensions);
    }

    pub fn pick_object(&mut self, x: f64, y: f64, ecs: &ECS, model_manager: &ModelManager) -> Option<Entity> {

        if !self.buf.is_some() {
            self.create_buffer(self.image.dimensions());
        }
        let hidpi_factor = self.surface.window().get_hidpi_factor();
        let x = (x * hidpi_factor).round() as usize;
        let y = (y * hidpi_factor).round() as usize;
        let buf_pos = 4 * (y * (self.dimensions[0] as usize) + x); //rgba

        let (view, proj) = ecs.camera.get_vp(); 

        let objs: Vec<_> =  ecs.components.models
            .iter()
            .zip(ecs.components.transforms.iter())
            .enumerate()
            .filter(|(_, (x, y))| x.is_some() && y.is_some())
            .map(|(i, (x, y))| {

                // entity ID, model and position
                (i,
                 x.as_ref().unwrap().value(),
                 y.as_ref().unwrap().value())

            }).collect();

        // Specify the color to clear the framebuffer with.
        // Important to have transparent color for color attachement as it means no
        // object.
        let clear_values = vec!([0.0, 0.0, 0.0, 0.0].into(), 1f32.into());

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(self.device.clone(), self.queue.family()).unwrap()
            .begin_render_pass(self.framebuffer.clone(), false, clear_values)
            .unwrap();

        for (id, model, transform) in objs.iter() {

            let model = model_manager.models.get(
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

            let push_constants = Object3DPicker::create_pushconstants(*id);

            command_buffer_builder = command_buffer_builder
                .draw_indexed(self.pipeline.pipeline.clone(),
                &DynamicState::none(),
                vec![model.vertex_buffer.clone()],
                model.index_buffer.clone(),
                set.clone(),
                push_constants).unwrap();
        }

        // Finish render pass
        command_buffer_builder = command_buffer_builder.end_render_pass()
            .unwrap();

        // Now, copy the image to the cpu accessible buffer
        command_buffer_builder = command_buffer_builder
            .copy_image_to_buffer(self.image.clone(),
            self.buf.as_ref().unwrap().clone()).unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = command_buffer_builder.build().unwrap();

        let finished = command_buffer.execute(self.queue.clone()).unwrap();
        finished.then_signal_fence_and_flush().unwrap()
            .wait(None).unwrap();

        let buffer_content = self.buf.as_ref().unwrap().read().unwrap();

        //  let image = ImageBuffer::<Rgba<u8>, _>::from_raw(
        //      self.dimensions[0], self.dimensions[1], &buffer_content[..]).unwrap();
        //  image.save("image.png").unwrap();

        // we have the index of the entity. Let's assume its alive as it shows up on
        // screen. We can then reconstruct the GenerationalIndex.
        let maybe_id = Object3DPicker::get_entity_id(
            buffer_content[buf_pos],
            buffer_content[buf_pos+1],
            buffer_content[buf_pos+2],
            buffer_content[buf_pos+3],
            );

        if let Some(id) = maybe_id {
            let gen = ecs.components.transforms[id].as_ref()?.generation();
            return Some(
                GenerationalIndex::new(
                    id,
                    gen,
                    ))
        }
        None
    }
}
