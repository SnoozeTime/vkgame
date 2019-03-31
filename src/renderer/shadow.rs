use cgmath::{InnerSpace, Matrix4, Point3, SquareMatrix, Vector3};
use log::*;
use std::sync::Arc;
use vulkano::buffer::{cpu_pool::CpuBufferPool, BufferUsage};
use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano::image::{AttachmentImage, ImageUsage};
use vulkano::pipeline::{viewport::Viewport, GraphicsPipeline, GraphicsPipelineAbstract};

use crate::ecs::components::{ModelComponent, TransformComponent};
use crate::event::{Event, ResourceEvent};
use crate::renderer::model::Vertex;
use crate::resource::Resources;

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

    pub fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2])
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

    pub fn shadow_map(&self) -> Arc<AttachmentImage> {
        self.shadow_map.clone()
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

    /// This is the first pass for rendering shadow. The scene will be written to the
    /// shadow map from the pov of the light
    pub fn draw_shadowmap(
        &self,
        resources: &Resources,
        light_transform: &TransformComponent,
        objects: &Vec<(&ModelComponent, &TransformComponent)>,
    ) -> AutoCommandBuffer {
        let (view, proj) = ShadowSystem::get_vp(light_transform);

        // 1. Create the secondary command buffer
        // --------------------------------------
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass(),
        )
        .unwrap();

        // 2. Draw all objects in the scene
        // --------------------------------
        for (model, transform) in objects.iter() {
            let model_buf = resources.models.models.get(&model.mesh_name);
            if !model_buf.is_some() {
                error!("Model {} is not loaded", model.mesh_name);
                continue;
            }
            let model = model_buf.unwrap();

            let uniform_buffer_subbuffer = {
                let uniform_data = create_mvp(transform, &view, &proj);
                self.uniform_buffer.next(uniform_data).unwrap()
            };

            let set = Arc::new(
                PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                    .add_buffer(uniform_buffer_subbuffer)
                    .unwrap()
                    .build()
                    .unwrap(),
            );
            // Now we can issue the draw command.
            builder = builder
                .draw_indexed(
                    self.pipeline.clone(),
                    &DynamicState::none(),
                    vec![model.vertex_buffer.clone()],
                    model.index_buffer.clone(),
                    set.clone(),
                    (),
                )
                .unwrap();
        }

        debug!("End render scene");
        debug!("----------------------------------------------");
        builder.build().unwrap()
    }

    /// Get the view matrix from the light. Projection is also needed and here
    /// we use an orthogonal projection
    fn get_vp(transform: &TransformComponent) -> (Matrix4<f32>, Matrix4<f32>) {
        let ortho = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let up = Vector3::new(0.0, 1.0, 0.0);
        // somewhere far away as it is a directional light.
        let position = -50.0
            * Point3::new(
                transform.position.x,
                transform.position.y,
                transform.position.z,
            );
        let view = Matrix4::look_at(position, position + transform.position, up);
        let mut the_fix = Matrix4::identity();
        the_fix[1][1] = -1.0;
        the_fix[2][3] = 0.5;
        the_fix[2][2] = 0.5;

        (view, the_fix * ortho)
    }
}

fn create_mvp(t: &TransformComponent, view: &Matrix4<f32>, proj: &Matrix4<f32>) -> vs::ty::Data {
    let scale = t.scale;
    let from_t = Matrix4::from_translation(t.position);
    let from_s = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
    trace!("From translation: {:?}", from_t);
    trace!("From scale: {:?}", from_s);
    let model = from_t * from_s;

    trace!("Model {:?}, View {:?}, Projection {:?}", model, view, proj);
    vs::ty::Data {
        model: model.into(),
        view: (*view).into(),
        proj: (*proj).into(),
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
