use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Queue;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::image::ImageViewAccess;
use vulkano::pipeline::blend::{AttachmentBlend, BlendFactor, BlendOp};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use cgmath::Vector3;

use crate::event::{Event, ResourceEvent};
use std::iter;
use std::sync::Arc;

use super::utils::{self, Vertex2d};
use super::GBufferComponent;

pub struct PointLightingSystem {
    queue: Arc<Queue>,
    dimensions: [u32; 2],
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex2d]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    vs: vs::Shader,
    fs: fs::Shader,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
}

impl PointLightingSystem {
    pub fn new<R>(queue: Arc<Queue>, subpass: Subpass<R>) -> Self
    where
        R: RenderPassAbstract + Send + Sync + 'static,
    {
        let (vertex_buffer, index_buffer) =
            utils::create_quad(queue.clone()).expect("Could not create quad buff");
        let vs = vs::Shader::load(queue.device().clone())
            .expect("Failed to create vertex shader module");
        let fs = fs::Shader::load(queue.device().clone())
            .expect("Failed to create fragment shader module");

        let pipeline =
            PointLightingSystem::build_pipeline(queue.clone(), subpass, [1, 1], &vs, &fs);

        PointLightingSystem {
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
        self.pipeline = PointLightingSystem::build_pipeline(
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
                .unwrap(),
        )
    }

    /// Draw the color added the light at position `position` and color `color`
    pub fn draw(
        &self,
        color_input: &GBufferComponent,
        normals_input: &GBufferComponent,
        frag_pos_input: &GBufferComponent,
        depth_input: &GBufferComponent,
        position: Vector3<f32>,
        color: [f32; 3],
    ) -> AutoCommandBuffer {
        // Data for the light source
        let push_constants = fs::ty::PushConstants {
            position: position.extend(0.0).into(),
            color: [color[0], color[1], color[2], 1.0],
        };

        // gbuffer. Input that was rendered in previous pass
        let descriptor_set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
            .add_sampled_image(color_input.image.clone(), color_input.sampler.clone())
            .unwrap()
            .add_sampled_image(normals_input.image.clone(), normals_input.sampler.clone())
            .unwrap()
            .add_sampled_image(frag_pos_input.image.clone(), frag_pos_input.sampler.clone())
            .unwrap()
            .add_sampled_image(depth_input.image.clone(), depth_input.sampler.clone())
            .unwrap()
            .build()
            .unwrap();

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
            descriptor_set,
            push_constants,
        )
        .unwrap()
        .build()
        .unwrap()
    }

    pub fn handle_event(&mut self, ev: &Event) {
        if let Event::ResourceEvent(ResourceEvent::ResourceReloaded(ref path)) = ev {
            if (*path).ends_with("point_light.vert") || (*path).ends_with("point_light.frag") {
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
        path: "assets/shaders/light/point_light.frag",
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
        push_constants: {
            name: PushConstants,
            ranges: [(color, 4), (position, 4)]
        },
        descriptors: [
            {
                name: u_diffuse,
                ty: SampledImage,
                set: 0,
                binding: 0
            },
            {
                name: u_normals,
                ty: SampledImage,
                set: 0,
                binding: 1
            },
            {
                name: u_frag_pos,
                ty: SampledImage,
                set: 0,
                binding: 2
            },
            {
                name: u_depth,
                ty: SampledImage,
                set: 0,
                binding: 3
            }
        ]
    }
}
