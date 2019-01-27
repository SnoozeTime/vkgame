use cgmath::{Point3, Vector3};
use std::sync::Arc;
use cgmath::{Matrix3, Matrix4, Rad};
use std::time::Duration;
use vulkano::command_buffer::{DrawIndexedError, DynamicState, AutoCommandBufferBuilder};
use vulkano::descriptor::descriptor_set::{DescriptorSet, PersistentDescriptorSet};

use crate::render::{vs, RenderSystem};
use crate::error::TwResult;
use crate::camera::Camera;


#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Point3<f32>,
}

pub struct MeshComponent {
    pub mesh_name: String,
    pub texture_name: String,
}

pub struct Scene {
    pub transforms: Transform,
    pub mesh_components: MeshComponent,
    pub camera: Camera,
}

impl Scene {

    pub fn render(&self,
                  dt: Duration,
                  cmd_buffer_builder: AutoCommandBufferBuilder, 
                  render_system: &RenderSystem) -> Result<AutoCommandBufferBuilder, DrawIndexedError> {
        // Get model from cache
        let model = render_system.model_manager.models.get(
            &self.mesh_components.mesh_name
        ).unwrap();


        // Get the texture from cache
        let texture = render_system.texture_manager.textures.get(
            &self.mesh_components.texture_name
        ).unwrap();
        
        // BUILD DESCRIPTOR SETS.
        //
        // 1. For texture
        let tex_set = Arc::new(PersistentDescriptorSet::start(render_system.pipeline.pipeline.clone(), 1)
                               .add_sampled_image(texture.texture.clone(), texture.sampler.clone()).unwrap()
                               .build().unwrap()
        );

        // 2. For uniform
        let mut sets: Vec<Arc<DescriptorSet + Sync + Send>> = Vec::new();
        {
            let uniform_buffer_subbuffer = {
                let uniform_data = create_mvp(dt, &self.camera);
                render_system.uniform_buffer.next(uniform_data).unwrap()
            };

            let set = Arc::new(PersistentDescriptorSet::start(render_system.pipeline.pipeline.clone(), 0)
                               .add_buffer(uniform_buffer_subbuffer).unwrap()
                               .build().unwrap()
            );

            sets.push(set);
        }


        // UPDATE MVP
        let mut sets: Vec<Arc<DescriptorSet + Sync + Send>> = Vec::new();
        {
            let uniform_buffer_subbuffer = {
                let uniform_data = create_mvp(dt, &self.camera);
                render_system.uniform_buffer.next(uniform_data).unwrap()
            };

            let set = Arc::new(PersistentDescriptorSet::start(render_system.pipeline.pipeline.clone(), 0)
                               .add_buffer(uniform_buffer_subbuffer).unwrap()
                               .build().unwrap()
            );

            sets.push(set);
        }


        cmd_buffer_builder.draw_indexed(render_system.pipeline.pipeline.clone(),
        &DynamicState::none(),
        vec![model.vertex_buffer.clone()],
        model.index_buffer.clone(),
        (sets[0].clone(), tex_set.clone()),
        ())
    }
}

fn create_mvp(elapsed_time: Duration, camera: &Camera) -> vs::ty::Data {
    let rotation = elapsed_time.as_secs() as f64 + elapsed_time.subsec_nanos() as f64 / 1_000_000_000.0;
    let rotation = Matrix3::from_angle_y(Rad(rotation as f32));            

    let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), 1.0, 0.01, 100.0);

    let view = camera.look_at();

    vs::ty::Data {
        model: Matrix4::from(rotation).into(),
        view: view.into(),
        proj: proj.into(),
    }


}


