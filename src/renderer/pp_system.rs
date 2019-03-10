use vulkano::device::Queue;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::pipeline::{GraphicsPipelineAbstract, GraphicsPipeline};
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::viewport::Viewport;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::command_buffer::DynamicState;

use std::iter;
use std::sync::Arc;
use super::GBufferComponent;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, uv);

pub struct PPSystem {
    queue: Arc<Queue>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    vs: vs::Shader,
    fs: fs::Shader,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
}

impl PPSystem {

    pub fn new<R>(queue: Arc<Queue>,
                  subpass: Subpass<R>,
                  ) -> Self
        where R: RenderPassAbstract + Send + Sync + 'static {

            // Quad that cover the full screen
            let vertex_buffer = {
                CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::all(),
                                               [
                                Vertex { position: [-1.0, -1.0], uv: [0.0, 0.0], },
                                Vertex { position: [-1.0, 1.0], uv: [0.0, 1.0] },
                                Vertex { position: [1.0, -1.0], uv: [1.0, 0.0]},
                                Vertex { position: [1.0, 1.0], uv: [1.0, 1.0] },
                                               ].iter().cloned()).expect("Failed to create buffer")
            };

            let index_buffer = {
                CpuAccessibleBuffer::from_iter(queue.device().clone(), BufferUsage::all(),
                                               [0, 1, 2, 2, 1, 3].iter().cloned()).unwrap()
            };

            let vs = vs::Shader::load(queue.device().clone())
                .expect("Failed to create vertex shader module");
            let fs = fs::Shader::load(queue.device().clone())
                .expect("Failed to create fragment shader module");

            let pipeline = PPSystem::build_pipeline(
                queue.clone(),
                subpass,
                [1, 1],
                &vs,
                &fs
            );
            PPSystem {
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

        self.pipeline = PPSystem::build_pipeline(
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
                 .render_pass(subpass)
                 .build(queue.device().clone())
                 .unwrap())
    }

    /// Draw the color added the light at position `position` and color `color`
    pub fn draw(&self,
                diffuse: &GBufferComponent) -> AutoCommandBuffer
    {

        let descriptor = Arc::new(
                PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_sampled_image(diffuse.image.clone(), diffuse.sampler.clone()).unwrap()
                .build().unwrap()
            );

        AutoCommandBufferBuilder::secondary_graphics(self.queue.device().clone(),
        self.queue.family(),
        self.pipeline.clone().subpass())
            .unwrap()
            .draw_indexed(self.pipeline.clone(),
            &DynamicState::none(),
            vec![self.vertex_buffer.clone()],
            self.index_buffer.clone(),
            descriptor.clone(),
            ())
            .unwrap().build().unwrap()

    }
}

mod vs {
    twgraph_shader::twshader!{
        kind: "vertex",
        path: "assets/shaders/post_processing/edge.vert",
        input: [
            {
                name: "position",
                format: R32G32Sfloat
            },
            {
                name: "uv",
                format: R32G32Sfloat
            }
        ],
        output: [
            {
                name: "outUv",
                format: R32G32Sfloat
            }
        ]   
    }
}

mod fs {
    twgraph_shader::twshader! {
        kind: "fragment",
        path: "assets/shaders/post_processing/edge.frag",
        input: [
            {
                name: "uv",
                format: R32G32Sfloat
            }
        ],
        output: [
            {
                name: "f_color",
                format: R32G32B32A32Sfloat
            }
        ],
        descriptors: [
            {
                name: diffuseSampler,
                ty: SampledImage,
                set: 0,
                binding: 0
            }
        ]
    }
}
