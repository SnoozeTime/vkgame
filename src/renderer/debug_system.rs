use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Queue;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

use super::utils::{self, Vertex2d};
use crate::event::{Event, ResourceEvent};
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use super::GBufferComponent;
use std::iter;
use std::sync::Arc;

/// Just display an image on screen
pub struct DebugSystem {
    queue: Arc<Queue>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex2d]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    vs: vs::Shader,
    fs: fs::Shader,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    dimensions: [u32; 2],
}

impl DebugSystem {
    pub fn new<R>(queue: Arc<Queue>, subpass: Subpass<R>) -> Self
    where
        R: RenderPassAbstract + Send + Sync + 'static,
    {
        // Quad that cover the full screen
        let (vertex_buffer, index_buffer) =
            utils::create_quad(queue.clone()).expect("Could not create quad buff");
        let vs = vs::Shader::load(queue.device().clone())
            .expect("Failed to create vertex shader module");
        let fs = fs::Shader::load(queue.device().clone())
            .expect("Failed to create fragment shader module");

        let pipeline = DebugSystem::build_pipeline(queue.clone(), subpass, [1, 1], &vs, &fs);
        DebugSystem {
            queue,
            vertex_buffer,
            index_buffer,
            vs,
            fs,
            pipeline,
            dimensions: [1, 1],
        }
    }

    pub fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2])
    where
        R: RenderPassAbstract + Send + Sync + 'static,
    {
        self.dimensions = dimensions;
        self.pipeline = DebugSystem::build_pipeline(
            self.queue.clone(),
            subpass,
            dimensions,
            &self.vs,
            &self.fs,
        );
    }

    fn build_pipeline<R>(
        queue: Arc<Queue>,
        subpass: Subpass<R>,
        dimensions: [u32; 2],
        vs: &vs::Shader,
        fs: &fs::Shader,
    ) -> Arc<GraphicsPipelineAbstract + Send + Sync>
    where
        R: RenderPassAbstract + Send + Sync + 'static,
    {
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex2d>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(subpass)
                .build(queue.device().clone())
                .unwrap(),
        )
    }

    /// Draw the color added the light at position `position` and color `color`
    pub fn draw(&self, diffuse: &GBufferComponent) -> AutoCommandBuffer {
        let descriptor = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                .add_sampled_image(diffuse.image.clone(), diffuse.sampler.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap()
        .draw_indexed(
            self.pipeline.clone(),
            &DynamicState::none(),
            vec![self.vertex_buffer.clone()],
            self.index_buffer.clone(),
            descriptor.clone(),
            (),
        )
        .unwrap()
        .build()
        .unwrap()
    }

    pub fn handle_event(&mut self, ev: &Event) {
        if let Event::ResourceEvent(ResourceEvent::ResourceReloaded(ref path)) = ev {
            if (*path).ends_with("quad.vert") || (*path).ends_with("quad.frag") {
                if let Err(err) = self
                    .vs
                    .recompile(self.queue.device().clone())
                    .and_then(|_| self.fs.recompile(self.queue.device().clone()))
                    .and_then(|_| {
                        self.rebuild_pipeline(self.pipeline.clone().subpass(), self.dimensions);
                        Ok(())
                    })
                {
                    dbg!(err);
                }
            }
        }
    }
}

mod vs {
    twgraph_shader::twshader! {
        kind: "vertex",
        path: "assets/shaders/debug/quad.vert",
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
        path: "assets/shaders/debug/quad.frag",
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
