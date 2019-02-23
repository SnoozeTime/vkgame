/// This is where the game objects are drawn to the scene.
/// The commands will be add to a secondary buffer.
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
use crate::ecs::components::{TransformComponent, ModelComponent, LightComponent};


pub struct SceneDrawSystem {
    queue: Arc<Queue>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vs: vs::Shader,
    fs: fs::Shader,

    uniform_buffer: CpuBufferPool<vs::ty::Data>,
    light_buffer: CpuBufferPool<fs::ty::Data>,
}

impl SceneDrawSystem {

    pub fn new<R>(queue: Arc<Queue>,
                  subpass: Subpass<R>,
                  dimensions: [u32; 2]) -> Self
        where R: RenderPassAbstract + Send + Sync + 'static
        {

            let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(queue.device().clone(), BufferUsage::all());
            let light_buffer = CpuBufferPool::<fs::ty::Data>::new(queue.device().clone(), BufferUsage::all());
            let vs = vs::Shader::load(queue.device().clone()).unwrap();
            let fs = fs::Shader::load(queue.device().clone()).unwrap();

            let pipeline = SceneDrawSystem::build_pipeline(
                queue.clone(),
                subpass,
                dimensions,
                &vs,
                &fs,
                );
                
                
            SceneDrawSystem {
                queue,
                pipeline,
                fs,
                vs,
                light_buffer,
                uniform_buffer,
            }
        } 

    pub fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2])
        where R: RenderPassAbstract + Send + Sync + 'static {

        self.pipeline = SceneDrawSystem::build_pipeline(
            self.queue.clone(),
            subpass,
            dimensions,
            &self.vs,
            &self.fs,
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
                     .viewports_dynamic_scissors_irrelevant(1)
                     .depth_stencil_simple_depth()
                     .viewports(iter::once(Viewport {
                         origin: [0.0, 0.0],
                         dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                         depth_range: 0.0 .. 1.0,
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
                lights: Vec<(&LightComponent, &TransformComponent)>,
                objects: Vec<(&ModelComponent, &TransformComponent)>) -> AutoCommandBuffer {

        let (view, proj) = camera.get_vp(); 
        // Clear to no color and infinite depth :)
        //let clear_values = vec!([0.0, 0.0, 0.0, 1.0].into(), 1f32.into());

        // Get light data
        let (color, position) = if lights.len() > 0 {
            let (light, transform) = lights[0];
            (light.color, transform.position.into())
        } else {
            ([0.5, 0.5, 0.5], [5.0, 0.5, 1.0])
        };

        // 1. Forward lighting
        // -------------------
        let light_buffer = {
            let data = fs::ty::Data {
                color,
                position,
                _dummy0: [0;4], // wtf is that?
            };
            self.light_buffer.next(data).unwrap()
        };


        // 2. Create the secondary command buffer
        // --------------------------------------
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass()).unwrap();

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
                .add_buffer(light_buffer.clone()).unwrap()
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
        path: "shaders/main.frag"
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


