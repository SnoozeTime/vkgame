use cgmath::{Vector3};
use std::sync::Arc;
use cgmath::Matrix4;
use vulkano::command_buffer::{DrawIndexedError, DynamicState, AutoCommandBufferBuilder};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};

use crate::render::{vs, RenderSystem};
use crate::camera::Camera;
use crate::ser::VectorDef;
use crate::error::TwResult;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {

    #[serde(with = "VectorDef")]
    pub position: Vector3<f32>,

    #[serde(with = "VectorDef")]
    pub rotation: Vector3<f32>,

    #[serde(with = "VectorDef")]
    pub scale: Vector3<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshComponent {
    pub mesh_name: String,
    pub texture_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scene {
    pub transforms: Vec<Transform>,
    pub mesh_components: Vec<MeshComponent>,
    pub camera: Camera,
}


impl Scene {

    pub fn new_dummy() -> Self {

        let mesh_components = vec![MeshComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        },MeshComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        }];


        let transform1 = Transform {
            position: Vector3::new(5.0, -2.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(0.0, 0.0, 0.0),
        };


        let transform2 = Transform {
            position: Vector3::new(0.0, -2.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(0.0, 0.0, 0.0),
        };

        let camera_transform = Transform {
            position: Vector3::new(0.0, 0.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(0.0, 0.0, 0.0),
        };
        let camera = Camera::new(camera_transform);


        Scene {
            transforms: vec![transform1, transform2],
            mesh_components,
            camera,
        }
    }

    pub fn render(&self,
                  mut cmd_buffer_builder: AutoCommandBufferBuilder, 
                  render_system: &RenderSystem) -> Result<AutoCommandBufferBuilder, DrawIndexedError> {

        // VP Matrix are the same for all game object.
        //
        let (view, proj) = self.camera.get_vp(); 

        // Get the texture from cache
        let texture = render_system.texture_manager.textures.get(
            &self.mesh_components[0].texture_name
        ).unwrap();

        // BUILD DESCRIPTOR SETS.
        //
        // 1. For texture
        let tex_set = Arc::new(PersistentDescriptorSet::start(render_system.pipeline.pipeline.clone(), 1)
                               .add_sampled_image(texture.texture.clone(), texture.sampler.clone()).unwrap()
                               .build().unwrap()
        );

        for i in 0..self.transforms.len() {

            let model = render_system.model_manager.models.get(
                &self.mesh_components[i].mesh_name
            ).unwrap();

            let uniform_buffer_subbuffer = {
                let uniform_data = create_mvp(&self.transforms[i], &view, &proj);
                render_system.uniform_buffer.next(uniform_data).unwrap()
            };

            let set = Arc::new(PersistentDescriptorSet::start(render_system.pipeline.pipeline.clone(), 0)
                               .add_buffer(uniform_buffer_subbuffer).unwrap()
                               .build().unwrap()
            );


            cmd_buffer_builder =  cmd_buffer_builder.draw_indexed(render_system.pipeline.pipeline.clone(),
            &DynamicState::none(),
            vec![model.vertex_buffer.clone()],
            model.index_buffer.clone(),
            (set.clone(), tex_set.clone()),
            ()).unwrap();
        }

        Ok(cmd_buffer_builder)
    }

    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> TwResult<()> {
       let mut file =  OpenOptions::new()
           .write(true)
           .truncate(true)
           .create(true)
           .open(path)?;

        let j = serde_json::to_string(self)?;
        write!(file, "{}", j)?;

        Ok(())
    }

    pub fn load<P: AsRef<std::path::Path>>(path: P) -> TwResult<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let scene = serde_json::from_str(&content).unwrap();
        Ok(scene)
    }
}

fn create_mvp(t: &Transform, view: &Matrix4<f32>, proj: &Matrix4<f32>) -> vs::ty::Data {
    let model = Matrix4::from_translation(t.position);
        
    vs::ty::Data {
        model: model.into(),
        view: (*view).into(),
        proj: (*proj).into(),
    }


}


