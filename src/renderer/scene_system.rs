use log::*;
use vulkano::buffer::BufferUsage;

use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;

use cgmath::Matrix4;
use std::iter;
use std::sync::Arc;

use crate::camera::Camera;
use crate::ecs::components::{ModelComponent, TransformComponent};
use crate::event::{Event, ResourceEvent};
use crate::renderer::model::Vertex;
use crate::resource::Resources;

pub struct SceneDrawSystem {
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vs: vs::Shader,
    fs: fs::Shader,
    dimensions: [u32; 2],

    uniform_buffer: CpuBufferPool<vs::ty::Data>,
}

impl SceneDrawSystem {
    pub fn new<R>(queue: Arc<Queue>, subpass: Subpass<R>, dimensions: [u32; 2]) -> Self
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        let uniform_buffer =
            CpuBufferPool::<vs::ty::Data>::new(queue.device().clone(), BufferUsage::all());
        let vs = vs::Shader::load(queue.device().clone()).unwrap();
        let fs = fs::Shader::load(queue.device().clone()).unwrap();

        let pipeline =
            SceneDrawSystem::build_pipeline(queue.clone(), subpass.clone(), dimensions, &vs, &fs);

        SceneDrawSystem {
            queue,

            pipeline,
            fs,
            vs,

            uniform_buffer,
            dimensions: [1, 1],
        }
    }

    pub fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2])
    where
        R: RenderPassAbstract + Clone + Send + Sync + 'static,
    {
        self.dimensions = dimensions;
        self.pipeline = SceneDrawSystem::build_pipeline(
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
        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<Vertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .cull_mode_back()
                .viewports_dynamic_scissors_irrelevant(1)
                .depth_stencil_simple_depth()
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

    /// Builds a secondary buffer that will draw all the scene object to the current
    /// subpass
    pub fn draw(
        &self,
        resources: &Resources,
        camera: &mut Camera,
        objects: &Vec<(&ModelComponent, &TransformComponent)>,
    ) -> AutoCommandBuffer {
        trace!("----------------------------------------------");
        trace!("begin scene rendering");
        trace!("Camera at position: {:?}", camera.state.transform.position);
        let (view, proj) = camera.get_vp();

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
            // I don't want to crash if the texture does not exist/is not loaded
            // just print a warning and do not render this object.
            let texture = resources.textures.textures.get(&model.texture_name);
            if !texture.is_some() {
                error!("Texture {} is not loaded", model.texture_name);
                continue;
            }
            let texture = texture.unwrap();

            // Same for the mesh
            let model_buf = resources.models.models.get(&model.mesh_name);
            if !model_buf.is_some() {
                error!("Model {} is not loaded", model.mesh_name);
                continue;
            }
            let model = model_buf.unwrap();

            // Create uniforms.
            // One is for the position,
            // Other is for fragment
            let uniform_buffer_subbuffer = {
                let uniform_data = create_mvp(transform, &view, &proj);
                self.uniform_buffer.next(uniform_data).unwrap()
            };
            trace!("Render object at position {:?}", transform);

            let set = Arc::new(
                PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                    .add_buffer(uniform_buffer_subbuffer)
                    .unwrap()
                    .build()
                    .unwrap(),
            );
            let tex_set = Arc::new(
                PersistentDescriptorSet::start(self.pipeline.clone(), 1)
                    .add_sampled_image(texture.texture.clone(), texture.sampler.clone())
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
                    (set.clone(), tex_set.clone()),
                    (),
                )
                .unwrap();
        }

        trace!("End render scene");
        trace!("----------------------------------------------");
        builder.build().unwrap()
    }

    pub fn handle_event(&mut self, ev: &Event) {
        if let Event::ResourceEvent(ResourceEvent::ResourceReloaded(ref path)) = ev {
            if (*path).ends_with("main.vert") || (*path).ends_with("deferred.frag") {
                println!("Recompiling skybox");
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

pub mod vs {
    twgraph_shader::twshader! {
        kind: "vertex",
        path: "assets/shaders/main.vert",
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
        output: [
            {
                name: "frag_color",
                format: R32G32B32A32Sfloat
            },
            {
                name: "frag_tex_coords",
                format: R32G32Sfloat
            },
            {
                name: "frag_position",
                format: R32G32B32Sfloat
            },
            {
                name: "frag_normal",
                format: R32G32B32Sfloat
            }
        ],
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
        path: "assets/shaders/deferred.frag",
        input: [
            {
                name: "frag_color",
                format: R32G32B32A32Sfloat
            },
            {
                name: "frag_tex_coords",
                format: R32G32Sfloat
            },
            {
                name: "frag_position",
                format: R32G32B32Sfloat
            },
            {
                name: "frag_normal",
                format: R32G32B32Sfloat
            }
        ],
        output: [
            {
                name: "f_color",
                format: R32G32B32A32Sfloat
            },
            {
                name: "f_normal",
                format: R32G32B32Sfloat
            },
            {
                name: "f_pos",
                format: R32G32B32Sfloat
            }
        ],
        descriptors: [
            {
                name: texSampler,
                ty: SampledImage,
                set: 1,
                binding: 0
            }
        ],
    }
}

pub fn create_mvp(
    t: &TransformComponent,
    view: &Matrix4<f32>,
    proj: &Matrix4<f32>,
) -> vs::ty::Data {
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
