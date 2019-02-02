use serde_derive::{Serialize, Deserialize};
use cgmath::Vector3;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read};

pub mod components;
pub mod systems;
use self::components::{TransformComponent, ModelComponent};
use crate::camera::Camera;
use crate::error::TwResult;

// TODO change to generational ID.
type EntityId = usize;
type EntityArray<T> = Vec<Option<T>>;


#[derive(Serialize, Deserialize, Debug)]
pub struct ECS {

    // Allllll my components.
    transform_components: EntityArray<TransformComponent>,
    model_components: EntityArray<ModelComponent>,


    // For now, static camera. TODO add it as a component.
    pub camera: Camera,
}

impl ECS {
    pub fn dummy_ecs() -> ECS {

        let model_components = vec![Some(ModelComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        }), Some(ModelComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        })];


        let transform1 = TransformComponent {
            position: Vector3::new(5.0, -2.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };


        let transform2 = TransformComponent {
            position: Vector3::new(0.0, -2.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };

        let camera_transform = TransformComponent {
            position: Vector3::new(0.0, 0.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };
        let camera = Camera::new(camera_transform);


        ECS {
            transform_components: vec![Some(transform1), Some(transform2)],
            model_components,
            camera,
        }
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
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let ecs = serde_json::from_str(&content).unwrap();
        Ok(ecs)
    }
}
