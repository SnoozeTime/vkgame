use std::sync::Arc;
use vulkano::buffer::{cpu_pool::CpuBufferPool, BufferUsage};
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::image::{AttachmentImage, ImageUsage};
use vulkano::pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract};

use crate::event::{Event, ResourceEvent};
use crate::renderer::model::Vertex;

/// Will render the shadows to the screen.
///
/// Shadow consists of two passes. First, the shadow maps
/// are rendered to depth buffers, from the perspective of the
/// light.
///
/// Then, during lighting, the shadow map will be used to find what
/// fragments are in the shadow. This system will just draw the first
/// pass to the depth buffer.
///
/// For now, only directional lights will cast shadows. And only one :D
pub struct ShadowSystem {
    queue: Arc<Queue>,
    dimensions: [u32; 2],

    // Our only shadow map for the moment. Will be used with the first directional
    // light.
    shadow_map: Arc<AttachmentImage>,

    // Shaders are simple for the first pass. The vertex shader renders the scene
    // from the point of vue of the light. Only the depth information is kept so
    // no need to render textures and no need for fragment shader.
    vs: vs::Shader,
    fs: fs::Shader,
    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
}

impl ShadowSystem {
    pub fn new<R>(queue: Arc<Queue>, subpass: Subpass<R>) -> Self
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        let uniform_buffer =
            CpuBufferPool::<vs::ty::Data>::new(queue.device().clone(), BufferUsage::all());
        let dimensions = [1, 1];
        let shadow_map = ShadowSystem::build_image(queue.device().clone(), dimensions);

        let vs = vs::Shader::load(queue.device().clone()).unwrap();
        let fs = fs::Shader::load(queue.device().clone()).unwrap();

        let pipeline = ShadowSystem::build_pipeline(queue.clone(), subpass, dimensions, &vs, &fs);
        ShadowSystem {
            queue,
            dimensions,

            vs,
            fs,
            uniform_buffer,
            pipeline,

            shadow_map,
        }
    }

    fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2])
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        self.dimensions = dimensions;
        self.shadow_map = ShadowSystem::build_image(self.queue.device().clone(), dimensions);
        self.pipeline = ShadowSystem::build_pipeline(
            self.queue.clone(),
            subpass.clone(),
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
        // Nothing special here. Just different shaders.
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(std::iter::once(Viewport {
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

    /// The shadow map is a depth buffer. It will be used in another subpass of the same
    /// render pass as input so it needs to have the correct usage flags.
    fn build_image(device: Arc<Device>, dimensions: [u32; 2]) -> Arc<AttachmentImage> {
        let usage = ImageUsage {
            input_attachment: true,
            ..ImageUsage::none()
        };

        AttachmentImage::with_usage(device, dimensions, Format::D16Unorm, usage).unwrap()
    }

    /// Rebuild the shaders
    pub fn handle_event(&mut self, ev: &Event) {
        if let Event::ResourceEvent(ResourceEvent::ResourceReloaded(ref path)) = ev {
            if (*path).ends_with("shadow.vert") || (*path).ends_with("shadow.frag") {
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
        path: "assets/shaders/light/shadow.vert",
        input: [
            {
                name: "position",
                format: R32G32B32Sfloat
            },
            {
                name: "texcoords",
                format: R32G32Sfloat
            },
            {
                name: "normals",
                format: R32G32B32Sfloat
            }
        ],
        output: [],
        descriptors: [
            {
                name: Data,
                ty: Buffer,
                set: 0,
                binding: 0,
                data: [
                    (model, "mat4"),
                    (view, "mat4"),
                    (proj, "mat4")
                ]
            }
        ]
    }
}

mod fs {
    twgraph_shader::twshader! {
        kind: "fragment",
        path: "assets/shaders/light/shadow.frag",
        input: [],
        output: []
    }
}
