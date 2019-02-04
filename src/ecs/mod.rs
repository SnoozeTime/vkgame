use serde_derive::{Serialize, Deserialize};
use cgmath::Vector3;
use std::fs::{OpenOptions, File};
use std::io::{Write, Read};

pub mod gen_index;
pub mod components;
pub mod systems;
use self::components::{TransformComponent, ModelComponent};
use self::gen_index::{GenerationalIndexAllocator, GenerationalIndexArray, GenerationalIndex};
use crate::camera::Camera;
use crate::error::TwResult;

type EntityArray<T> = GenerationalIndexArray<T>;

#[derive(Serialize, Deserialize, Debug)]
pub struct ECS {

    // Allllll my components.
    transform_components: EntityArray<TransformComponent>,
    model_components: EntityArray<ModelComponent>,

    #[serde(skip)]
    #[serde(default = "GenerationalIndexAllocator::new")]
    allocator: GenerationalIndexAllocator,
    // For now, static camera. TODO add it as a component.
    //
    #[serde(skip)]
    pub camera: Camera,
}

impl ECS {

    pub fn new() -> Self {
        // ----------
        let model_components = GenerationalIndexArray(Vec::new());
        let transform_components = GenerationalIndexArray(Vec::new());

        // ----------
        let camera = Camera::default();

        ECS {
            transform_components,
            model_components,
            camera,
            allocator: GenerationalIndexAllocator::new(),
        }
    }

    pub fn new_entity(&mut self) -> GenerationalIndex {
        let index = self.allocator.allocate();

        // Check if index equal size of arrays. If yes, extend all of them!.
        if index.index() == self.transform_components.len() {
            self.model_components.push(None);
            self.transform_components.push(None);
        } else {

            // put None everywhere :D
            self.model_components.empty(&index);
            self.transform_components.empty(&index);
        }

        index
    }

    pub fn dummy_ecs() -> ECS {

        let mut ecs = ECS::new();

        // First entity
        let id1 = ecs.new_entity();
        ecs.transform_components.set(&id1, TransformComponent {
            position: Vector3::new(0.0, 0.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        ecs.model_components.set(&id1, ModelComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        });

        // Second entity
        let id2 = ecs.new_entity();
        ecs.transform_components.set(&id2, TransformComponent {
            position: Vector3::new(1.0, -2.0, 4.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(5.0, 2.0, 4.0),
        });
        ecs.model_components.set(&id2, ModelComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "white".to_owned(),
        });
        let id3 = ecs.new_entity();
        ecs.transform_components.set(&id3, TransformComponent {
            position: Vector3::new(1.0, 0.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        ecs.model_components.set(&id3, ModelComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        });
let id4 = ecs.new_entity();
        ecs.transform_components.set(&id4, TransformComponent {
            position: Vector3::new(1.0, 1.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        ecs.model_components.set(&id4, ModelComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        });


        ecs
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
        println!("ECS after loading{:?}", ecs);
        Ok(ecs)
    }
}
