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

use std::iter;
use std::sync::Arc;

use crate::event::{Event, ResourceEvent};

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

/// Render light that comes from infinity from a certain direction.
/// A directional light can cast shadows. In that case, the shadow map
/// needs to be rendered by the shadow system
pub struct DirectionalLightingSystem {
    queue: Arc<Queue>,

    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    vs: vs::Shader,
    fs: fs::Shader,
    shadow_fs: shadow_fs::Shader,
    // normal lighting without casting shadows.
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    // lighting with shadows.
    shadow_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,

    dimensions: [u32; 2],
}

impl DirectionalLightingSystem {
    pub fn new<R>(queue: Arc<Queue>, subpass: Subpass<R>) -> Self
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        // that is suspicious
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                queue.device().clone(),
                BufferUsage::all(),
                [
                    Vertex {
                        position: [-1.0, -1.0],
                    },
                    Vertex {
                        position: [-1.0, 3.0],
                    },
                    Vertex {
                        position: [3.0, -1.0],
                    },
                ]
                .iter()
                .cloned(),
            )
            .expect("Failed to create buffer")
        };

        let vs = vs::Shader::load(queue.device().clone())
            .expect("Failed to create vertex shader module");
        let fs = fs::Shader::load(queue.device().clone())
            .expect("Failed to create fragment shader module");
        let shadow_fs = shadow_fs::Shader::load(queue.device().clone())
            .expect("Failed to create shadow fragment shader module");

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
            vs,
            fs,
            shadow_fs,
            pipeline,
            shadow_pipeline,
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
                    .vertex_input_single_buffer::<Vertex>()
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
                    .vertex_input_single_buffer::<Vertex>()
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
    pub fn draw<C, N, D>(
        &self,
        color_input: C,
        normals_input: N,
        depth_input: D,
        direction: Vector3<f32>,
        color: [f32; 3],
    ) -> AutoCommandBuffer
    where
        C: ImageViewAccess + Send + Sync + 'static,
        N: ImageViewAccess + Send + Sync + 'static,
        D: ImageViewAccess + Send + Sync + 'static,
    {
        // Data for the light source
        let push_constants = fs::ty::PushConstants {
            position: direction.extend(0.0).into(),
            color: [color[0], color[1], color[2], 1.0],
        };

        // gbuffer. Input that was rendered in previous pass
        let descriptor_set = PersistentDescriptorSet::start(self.pipeline.clone(), 0)
            .add_image(color_input)
            .unwrap()
            .add_image(normals_input)
            .unwrap()
            .add_image(depth_input)
            .unwrap()
            .build()
            .unwrap();

        AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap()
        .draw(
            self.pipeline.clone(),
            &DynamicState::none(),
            vec![self.vertex_buffer.clone()],
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
    pub fn draw_with_shadow<C, N, D, SM>(
        &self,
        color_input: C,
        normals_input: N,
        depth_input: D,
        shadow_map: SM,
        direction: Vector3<f32>,
        color: [f32; 3],
    ) -> AutoCommandBuffer
    where
        C: ImageViewAccess + Send + Sync + 'static,
        N: ImageViewAccess + Send + Sync + 'static,
        D: ImageViewAccess + Send + Sync + 'static,
        SM: ImageViewAccess + Send + Sync + 'static,
    {
        // Data for the light source
        let push_constants = fs::ty::PushConstants {
            position: direction.extend(0.0).into(),
            color: [color[0], color[1], color[2], 1.0],
        };

        // gbuffer. Input that was rendered in previous pass
        let descriptor_set = PersistentDescriptorSet::start(self.shadow_pipeline.clone(), 0)
            .add_image(color_input)
            .unwrap()
            .add_image(normals_input)
            .unwrap()
            .add_image(depth_input)
            .unwrap()
            .add_image(shadow_map)
            .unwrap()
            .build()
            .unwrap();

        AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.shadow_pipeline.clone().subpass(),
        )
        .unwrap()
        .draw(
            self.shadow_pipeline.clone(),
            &DynamicState::none(),
            vec![self.vertex_buffer.clone()],
            descriptor_set,
            push_constants,
        )
        .unwrap()
        .build()
        .unwrap()
    }

    pub fn handle_event(&mut self, ev: &Event) {
        if let Event::ResourceEvent(ResourceEvent::ResourceReloaded(ref path)) = ev {
            if (*path).ends_with("directional_light.vert")
                || (*path).ends_with("directional_light.frag")
            {
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
        path: "assets/shaders/light/directional_light.vert",
        input: [
        {
            name: "position",
            format: R32G32Sfloat
        }
        ],
        output: [
        {
            name: "v_screen_coords",
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
            name: "v_screen_coords",
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
            ty: InputAttachment,
            set: 0,
            binding: 0
        },
        {
            name: u_normals,
            ty: InputAttachment,
            set: 0,
            binding: 1
        },
        {
            name: U_depth,
            ty: InputAttachment,
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
            name: "v_screen_coords",
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
            ty: InputAttachment,
            set: 0,
            binding: 0
        },
        {
            name: u_normals,
            ty: InputAttachment,
            set: 0,
            binding: 1
        },
        {
            name: u_depth,
            ty: InputAttachment,
            set: 0,
            binding: 2
        },
        {
            name: u_shadow,
            ty: InputAttachment,
            set: 0,
            binding: 3
        }
        ]
    }
}
