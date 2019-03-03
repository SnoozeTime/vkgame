use vulkano::device::Queue;
use vulkano::image::ImageViewAccess;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::blend::{AttachmentBlend, BlendFactor, BlendOp};
use vulkano::pipeline::viewport::Viewport;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::command_buffer::DynamicState;

use cgmath::Vector3;

use std::iter;
use std::sync::Arc;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, uv);

use crate::renderer::GBufferComponent;

pub struct PointLightingSystem {

    queue: Arc<Queue>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    vs: vs::Shader,
    fs: fs::Shader,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
}


impl PointLightingSystem {

    pub fn new<R>(queue: Arc<Queue>,
                  subpass: Subpass<R>) -> Self
        where R: RenderPassAbstract + Send + Sync + 'static {

            // vertices
            // Quad that cover the full screen
            let vertex_buffer = {
                CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::all(),
                                               [
                                               Vertex { position: [-1.0, -1.0], uv: [0.0, 0.0]},
                                               Vertex { position: [-1.0, 1.0], uv: [0.0, 1.0]},
                                               Vertex { position: [1.0, -1.0], uv: [1.0, 0.0]},
                                               Vertex { position: [1.0, 1.0], uv: [1.0, 1.0]},
                                               ].iter().cloned()).expect("Failed to create buffer")
            };

            let index_buffer = {
                CpuAccessibleBuffer::from_iter(queue.device().clone(),
                BufferUsage::all(), [0, 1, 2, 2, 3, 1].iter().cloned()).unwrap()
            };


            let vs = vs::Shader::load(queue.device().clone())
                .expect("Failed to create vertex shader module");
            let fs = fs::Shader::load(queue.device().clone())
                .expect("Failed to create fragment shader module");

            let pipeline = PointLightingSystem::build_pipeline(
                queue.clone(),
                subpass,
                [1, 1],
                &vs,
                &fs
            );

            PointLightingSystem {
                queue,
                vertex_buffer,
                index_buffer,
                vs,
                fs,
                pipeline,
            }
        }

    pub fn rebuild_pipeline<R>(
        &mut self,
        subpass: Subpass<R>,
        dimensions: [u32; 2],
        ) where R: RenderPassAbstract + Send + Sync + 'static {

        self.pipeline = PointLightingSystem::build_pipeline(
            self.queue.clone(),
            subpass,
            dimensions,
            &self.vs,
            &self.fs);
    }


    fn build_pipeline<R>(
        queue: Arc<Queue>,
        subpass: Subpass<R>,
        dimensions: [u32; 2],
        vs: &vs::Shader,
        fs: &fs::Shader,
        ) -> Arc<GraphicsPipelineAbstract + Send + Sync> where R: RenderPassAbstract + Send + Sync + 'static {


        Arc::new(GraphicsPipeline::start()
                 .vertex_input_single_buffer::<Vertex>()
                 .vertex_shader(vs.main_entry_point(), ())
                 .triangle_list()
                 .viewports_dynamic_scissors_irrelevant(1)
                 .viewports(iter::once(Viewport {
                     origin: [0.0, 0.0],
                     dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                     depth_range: 0.0 .. 1.0,
                 }))
                 .fragment_shader(fs.main_entry_point(), ())
                 .blend_collective(AttachmentBlend {
                     enabled: true,
                     color_op: BlendOp::Add,
                     color_source: BlendFactor::One,
                     color_destination: BlendFactor::One,
                     alpha_op: BlendOp::Max,
                     alpha_source: BlendFactor::One,
                     alpha_destination: BlendFactor::One,
                     mask_red: true,
                     mask_green: true,
                     mask_blue: true,
                     mask_alpha: true,
                 })
        .render_pass(subpass)
            .build(queue.device().clone())
            .unwrap())
    }


    /// Draw the color added the light at position `position` and color `color`
    pub fn draw(&self,
                color_input: &GBufferComponent,
                normals_input: &GBufferComponent,
                frag_pos_input: &GBufferComponent,
                depth_input: &GBufferComponent,
                position: Vector3<f32>,
                color: [f32; 3]) -> AutoCommandBuffer
    {
        // Data for the light source
        let push_constants = fs::ty::PushConstants {
            position: position.extend(0.0).into(),
            color: [color[0], color[1], color[2], 1.0],
        };

        // gbuffer. Input that was rendered in previous pass
        let descriptor_set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
            .add_sampled_image(color_input.image.clone(), color_input.sampler.clone())
            .unwrap()
//            .add_sampled_image(normals_input.image.clone(),
//            normals_input.sampler.clone()).unwrap()
//            .add_sampled_image(frag_pos_input.image.clone(),
//            frag_pos_input.sampler.clone()).unwrap()
            .add_sampled_image(depth_input.image.clone(),
            depth_input.sampler.clone()).unwrap()
            .build().unwrap();

        AutoCommandBufferBuilder::secondary_graphics(self.queue.device().clone(),
        self.queue.family(),
        self.pipeline.clone().subpass())
            .unwrap()
            .draw_indexed(self.pipeline.clone(),
            &DynamicState::none(),
            vec![self.vertex_buffer.clone()],
            self.index_buffer.clone(),
            descriptor_set,
            push_constants)
            .unwrap().build().unwrap()

    }
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/quad.vert"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/point_light.frag"
    }
}


