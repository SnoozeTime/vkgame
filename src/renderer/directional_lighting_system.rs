use log::*;
use vulkano::buffer::{cpu_pool::CpuBufferPool, BufferUsage, CpuAccessibleBuffer};
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

use std::iter;
use std::sync::Arc;

use super::shadow::ShadowSystem;
use super::utils::{self, Vertex2d};
use super::GBufferComponent;
use crate::ecs::components::TransformComponent;
use crate::event::{Event, ResourceEvent};

/// Render light that comes from infinity from a certain direction.
/// A directional light can cast shadows. In that case, the shadow map
/// needs to be rendered by the shadow system
pub struct DirectionalLightingSystem {
    queue: Arc<Queue>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex2d]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,

    vs: vs::Shader,
    fs: fs::Shader,
    shadow_fs: shadow_fs::Shader,
    // normal lighting without casting shadows.
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    // lighting with shadows.
    shadow_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    uniform_buffer: CpuBufferPool<shadow_fs::ty::Data>,

    dimensions: [u32; 2],
}

impl DirectionalLightingSystem {
    pub fn new<R>(queue: Arc<Queue>, subpass: Subpass<R>) -> Self
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        let (vertex_buffer, index_buffer) =
            utils::create_quad(queue.clone()).expect("Could not create quad buff");

        let vs = vs::Shader::load(queue.device().clone())
            .expect("Failed to create vertex shader module");
        let fs = fs::Shader::load(queue.device().clone())
            .expect("Failed to create fragment shader module");
        let shadow_fs = shadow_fs::Shader::load(queue.device().clone())
            .expect("Failed to create shadow fragment shader module");
        let uniform_buffer =
            CpuBufferPool::<shadow_fs::ty::Data>::new(queue.device().clone(), BufferUsage::all());

        let (pipeline, shadow_pipeline) = DirectionalLightingSystem::build_pipeline(
            queue.clone(),
            subpass,
            [1, 1],
            &vs,
            &fs,
            &shadow_fs,
        );

        DirectionalLightingSystem {
            queue,
            vertex_buffer,
            index_buffer,
            vs,
            fs,
            shadow_fs,
            pipeline,
            shadow_pipeline,
            uniform_buffer,
            dimensions: [1, 1],
        }
    }

    pub fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2])
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        self.dimensions = dimensions;
        let (pipeline, shadow_pipeline) = DirectionalLightingSystem::build_pipeline(
            self.queue.clone(),
            subpass,
            dimensions,
            &self.vs,
            &self.fs,
            &self.shadow_fs,
        );
        self.pipeline = pipeline;
        self.shadow_pipeline = shadow_pipeline;
    }

    /// Two pipelines:
    ///  - one to render the lighting without casting shadows
    ///  - one to render the lighting with shadows
    fn build_pipeline<R>(
        queue: Arc<Queue>,
        subpass: Subpass<R>,
        dimensions: [u32; 2],
        vs: &vs::Shader,
        fs: &fs::Shader,
        shadow_fs: &shadow_fs::Shader,
    ) -> (
        Arc<GraphicsPipelineAbstract + Send + Sync>,
        Arc<GraphicsPipelineAbstract + Send + Sync>,
    )
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        (
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
                    .render_pass(subpass.clone())
                    .build(queue.device().clone())
                    .unwrap(),
            ),
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
                    .fragment_shader(shadow_fs.main_entry_point(), ())
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
            ),
        )
    }

    /// Draw the color added the light at position `position` and color `color`
    pub fn draw(
        &self,
        color_input: &GBufferComponent,
        normals_input: &GBufferComponent,
        depth_input: &GBufferComponent,
        direction: Vector3<f32>,
        color: [f32; 3],
    ) -> AutoCommandBuffer {
        // Data for the light source
        let push_constants = fs::ty::PushConstants {
            position: direction.extend(0.0).into(),
            color: [color[0], color[1], color[2], 1.0],
        };

        // gbuffer. Input that was rendered in previous pass
        let descriptor_set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
            .add_sampled_image(color_input.image.clone(), color_input.sampler.clone())
            .unwrap()
            .add_sampled_image(normals_input.image.clone(), normals_input.sampler.clone())
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

    /// Draw the color added the light at position `position` and color `color`
    /// With the shadow map.
    /// The shadow map SHOULD correspond to the light. otherwise strange things will happen :D
    pub fn draw_with_shadow(
        &self,
        color_input: &GBufferComponent,
        normals_input: &GBufferComponent,
        depth_input: &GBufferComponent,
        position_input: &GBufferComponent,
        shadow_map: &GBufferComponent,
        light_transform: &TransformComponent,
        color: [f32; 3],
    ) -> AutoCommandBuffer {
        debug!("Drawing shadows");
        // Data for the light source
        let direction: Vector3<f32> = light_transform.position.into();
        let push_constants = fs::ty::PushConstants {
            position: direction.extend(0.0).into(),
            color: [color[0], color[1], color[2], 1.0],
        };

        // View projection for the light.
        let (v, p) = ShadowSystem::get_vp(light_transform);
        let uniform_light_data = self
            .uniform_buffer
            .next(shadow_fs::ty::Data {
                view: v.into(),
                proj: p.into(),
            })
            .unwrap();

        debug!("View: {:?}", v);
        debug!("proj: {:?}", p);
        debug!(" P V {:?}", p * v);
        // gbuffer. Input that was rendered in previous pass + Some stuff for shadows.
        let descriptor_set = PersistentDescriptorSet::start(self.shadow_pipeline.clone(), 0)
            .add_sampled_image(color_input.image.clone(), color_input.sampler.clone())
            .unwrap()
            .add_sampled_image(normals_input.image.clone(), normals_input.sampler.clone())
            .unwrap()
            .add_sampled_image(depth_input.image.clone(), depth_input.sampler.clone())
            .unwrap()
            .add_sampled_image(position_input.image.clone(), position_input.sampler.clone())
            .unwrap()
            .add_sampled_image(shadow_map.image.clone(), shadow_map.sampler.clone())
            .unwrap()
            .add_buffer(uniform_light_data)
            .unwrap()
            .build()
            .unwrap();

        AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.shadow_pipeline.clone().subpass(),
        )
        .unwrap()
        .draw_indexed(
            self.shadow_pipeline.clone(),
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
            if (*path).ends_with("quad.vert")
                || (*path).ends_with("directional_light.frag")
                || (*path).ends_with("directional_light_shadow.frag")
            {
                if let Err(err) = self
                    .vs
                    .recompile(self.queue.device().clone())
                    .and_then(|_| self.fs.recompile(self.queue.device().clone()))
                    .and_then(|_| self.shadow_fs.recompile(self.queue.device().clone()))
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
        path: "assets/shaders/light/directional_light.frag",
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
            name: U_depth,
            ty: SampledImage,
            set: 0,
            binding: 2
        }
        ]
    }
}

mod shadow_fs {
    twgraph_shader::twshader! {
        kind: "fragment",
        path: "assets/shaders/light/directional_light_shadow.frag",
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
            name: u_depth,
            ty: SampledImage,
            set: 0,
            binding: 2
        },
        {
            name: u_position,
            ty: SampledImage,
            set: 0,
            binding: 3,
        },
        {
            name: u_shadow,
            ty: SampledImage,
            set: 0,
            binding: 4
        },
        {
            name: Data,
            ty: Buffer,
            set: 0,
            binding: 5,
            data: [
                (view, "mat4"),
                (proj, "mat4")
            ]
        }
        ]
    }
}
