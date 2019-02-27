use vulkano::buffer::BufferUsage;

use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use vulkano::device::Queue;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::viewport::Viewport;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use cgmath::Matrix4;
use std::sync::Arc;
use std::iter;

use crate::renderer::model::Vertex;
use crate::resource::Resources;
use crate::camera::Camera;
use crate::ecs::components::{TransformComponent, ModelComponent};

/// Will draw the current scene to the screen. Here, I will use
/// two passes (not vulkan passes) to create cel shading:
/// - First, draw the objects a big bigger in black for the outline
/// - Draw normal objects on top
///
/// Need two pipelines for that:
/// - Outline one will cull front faces
/// - Normal one will cull back faces
pub struct SceneDrawSystem {
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vs: vs::Shader,
    fs: fs::Shader,

    outline_pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    outline_vs: outline_vs::Shader,
    outline_fs: outline_fs::Shader,

    uniform_buffer: CpuBufferPool<vs::ty::Data>,
}

impl SceneDrawSystem {

    pub fn new<R>(queue: Arc<Queue>,
                  subpass: Subpass<R>,
                  dimensions: [u32; 2]) -> Self
        where R: RenderPassAbstract + Clone + Send + Sync + 'static
        {

            let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(queue.device().clone(), BufferUsage::all());
            let vs = vs::Shader::load(queue.device().clone()).unwrap();
            let fs = fs::Shader::load(queue.device().clone()).unwrap();

            let pipeline = SceneDrawSystem::build_pipeline(
                queue.clone(),
                subpass.clone(),
                dimensions,
                &vs,
                &fs,
                );


            let outline_vs = outline_vs::Shader::load(queue.device().clone()).unwrap();
            let outline_fs = outline_fs::Shader::load(queue.device().clone()).unwrap();
            let outline_pipeline = SceneDrawSystem::build_outline_pipeline(
                queue.clone(),
                subpass,
                dimensions,
                &outline_vs,
                &outline_fs,
                );

            SceneDrawSystem {
                queue,

                pipeline,
                fs,
                vs,

                outline_pipeline,
                outline_vs,
                outline_fs,

                uniform_buffer,
            }
        } 

    pub fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2])
        where R: RenderPassAbstract + Clone + Send + Sync + 'static {

            self.pipeline = SceneDrawSystem::build_pipeline(
                self.queue.clone(),
                subpass.clone(),
                dimensions,
                &self.vs,
                &self.fs,
                );

            self.outline_pipeline = SceneDrawSystem::build_outline_pipeline(
                self.queue.clone(),
                subpass,
                dimensions,
                &self.outline_vs,
                &self.outline_fs,
                );
        }

    fn build_pipeline<R>(queue: Arc<Queue>,
                         subpass: Subpass<R>,
                         dimensions: [u32; 2],
                         vs: &vs::Shader,
                         fs: &fs::Shader) -> Arc<GraphicsPipelineAbstract + Send + Sync> 
        where R: RenderPassAbstract + Send + Sync + 'static {
            Arc::new(GraphicsPipeline::start()
                     .vertex_input_single_buffer::<Vertex>()
                     .vertex_shader(vs.main_entry_point(), ())
                     .triangle_list()
                     .cull_mode_back()
                     .viewports_dynamic_scissors_irrelevant(1)
                     .depth_stencil_simple_depth()
                     .viewports(iter::once(Viewport {
                         origin: [0.0, 0.0],
                         dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                         depth_range: 0.0 .. 0.99,
                     }))
                     .fragment_shader(fs.main_entry_point(), ())
                     .render_pass(subpass)
                     .build(queue.device().clone()).unwrap())
        }

    fn build_outline_pipeline<R>(queue: Arc<Queue>,
                                 subpass: Subpass<R>,
                                 dimensions: [u32; 2],
                                 vs: &outline_vs::Shader,
                                 fs: &outline_fs::Shader) -> Arc<GraphicsPipelineAbstract + Send + Sync> 
        where R: RenderPassAbstract + Send + Sync + 'static {
            Arc::new(GraphicsPipeline::start()
                     .vertex_input_single_buffer::<Vertex>()
                     .vertex_shader(vs.main_entry_point(), ())
                     .triangle_list()
                     .cull_mode_front() // Changes here
                     .viewports_dynamic_scissors_irrelevant(1)
                     .depth_stencil_simple_depth()
                     .viewports(iter::once(Viewport {
                         origin: [0.0, 0.0],
                         dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                         depth_range: 0.0 .. 0.99,
                     }))
                     .fragment_shader(fs.main_entry_point(), ())
                     .render_pass(subpass)
                     .build(queue.device().clone()).unwrap())
        }


    /// Builds a secondary buffer that will draw all the scene object to the current
    /// subpass
    pub fn draw(&self,
                resources: &Resources,
                camera: &mut Camera,
                objects: Vec<(&ModelComponent, &TransformComponent)>) -> AutoCommandBuffer {

        let (view, proj) = camera.get_vp(); 

        // 1. Create the secondary command buffer
        // --------------------------------------
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass()).unwrap();

        // 2. Draw all outlines in scene
        // -----------------------------
        for (model, transform) in objects.iter() {

            let model_buf = resources.models.models.get(&model.mesh_name);
            if !model_buf.is_some() {
                println!("Model {} is not loaded", model.mesh_name);
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

            let set = Arc::new(PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                               .add_buffer(uniform_buffer_subbuffer).unwrap()
                               .build().unwrap()
            );
        
            // Now we can issue the draw command.
            builder = builder.draw_indexed(self.outline_pipeline.clone(),
            &DynamicState::none(),
            vec![model.vertex_buffer.clone()],
            model.index_buffer.clone(),
            set.clone(),
            ()).unwrap();
        }


        // 3. Draw all objects in the scene
        // --------------------------------
        for (model, transform) in objects.iter() {

            // I don't want to crash if the texture does not exist/is not loaded
            // just print a warning and do not render this object.
            let texture = resources.textures.textures.get(&model.texture_name);
            if !texture.is_some() {
                println!("Texture {} is not loaded", model.texture_name);
                continue;
            }
            let texture = texture.unwrap();

            // Same for the mesh
            let model_buf = resources.models.models.get(&model.mesh_name);
            if !model_buf.is_some() {
                println!("Model {} is not loaded", model.mesh_name);
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

            let set = Arc::new(PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                               .add_buffer(uniform_buffer_subbuffer).unwrap()
                               .build().unwrap()
            );
            let tex_set = Arc::new(
                PersistentDescriptorSet::start(self.pipeline.clone(), 1)
                .add_sampled_image(texture.texture.clone(), texture.sampler.clone()).unwrap()
                .build().unwrap()
            );

            // Now we can issue the draw command.
            builder = builder.draw_indexed(self.pipeline.clone(),
            &DynamicState::none(),
            vec![model.vertex_buffer.clone()],
            model.index_buffer.clone(),
            (set.clone(), tex_set.clone()),
            ()).unwrap();
        }


        builder.build().unwrap()
    }
}

pub mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/main.vert"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/deferred.frag"
    }
}

mod outline_vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/outline.vert"
    }
}

mod outline_fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/outline.frag"
    }
}


pub fn create_mvp(t: &TransformComponent, view: &Matrix4<f32>, proj: &Matrix4<f32>) -> vs::ty::Data {
    let scale = t.scale;
    let model = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
        * Matrix4::from_translation(t.position);


    vs::ty::Data {
        model: model.into(),
        view: (*view).into(),
        proj: (*proj).into(),
    }
}
