use vulkano::pipeline::viewport::Viewport;
use vulkano::device::{Device, Queue};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::impl_vertex;
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer, DynamicState};
use vulkano::swapchain::Surface;
use std::error::Error;
use vulkano::image::{Dimensions, ImageUsage, ImmutableImage, StorageImage};
use vulkano::format::{Format, R8G8B8A8Unorm};
use vulkano::sync::GpuFuture;
use vulkano::image::attachment::AttachmentImage;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use std::sync::Arc;
use std::iter;

use crate::ecs::ECS;
use super::model::{ModelManager, Vertex};

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

    pub pipeline: PickPipelineState,
    pub framebuffer: Arc<FramebufferAbstract + Send + Sync>,
    pub image: Arc<AttachmentImage>,
}


impl Object3DPicker {

    pub fn new(device: Arc<Device>,
               queue: Arc<Queue>,
               render_pass: Arc<RenderPassAbstract + Send + Sync>,
               dimensions: [u32; 2]) -> Self {

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
            pipeline,
            framebuffer,
            image,
        }
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
                            device: Arc<Device>,
                            render_pass: Arc<RenderPassAbstract + Send + Sync>,
                            dimensions: [u32; 2]) {
        let usage = ImageUsage {
            transfer_source: true,
            .. ImageUsage::none()
        };
        self.image = AttachmentImage::with_usage(
            device.clone(), 
            dimensions,
            Format::B8G8R8A8Srgb,
            usage).unwrap();
        let depth_buffer = AttachmentImage::transient(device.clone(),
        dimensions,
        Format::D16Unorm).unwrap();


        self.framebuffer = Arc::new(Framebuffer::start(render_pass.clone())
                                    .add(self.image.clone()).unwrap()
                                    .add(depth_buffer.clone()).unwrap()
                                    .build().unwrap());

        self.pipeline.rebuild_pipeline(device, render_pass, dimensions);
    }
}
