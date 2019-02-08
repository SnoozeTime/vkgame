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

    allocator: GenerationalIndexAllocator,
    // For now, static camera. TODO add it as a component.
    //
    #[serde(skip)]
    pub camera: Camera,

    components: Components,
}

impl ECS {

    pub fn new() -> Self {
        // ----------
        let camera = Camera::default();

        ECS {
            camera,
            allocator: GenerationalIndexAllocator::new(),
            components: Components::new(),
        }
    }

    pub fn new_entity(&mut self) -> GenerationalIndex {
        let index = self.allocator.allocate();
        self.components.new_entity(&index);
        index
    }

    pub fn dummy_ecs() -> ECS {

        let mut ecs = ECS::new();

        // First entity
        let id1 = ecs.new_entity();
        let components = &mut ecs.components;
        components.transforms.set(&id1, TransformComponent {
            position: Vector3::new(0.0, 0.0, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        });
        components.models.set(&id1, ModelComponent {
            mesh_name: "cube".to_owned(),
            texture_name: "bonjour".to_owned(),
        });

//        // Second entity
//        let id2 = ecs.new_entity();
//        transform_components.set(&id2, TransformComponent {
//            position: Vector3::new(1.0, -2.0, 4.0),
//            rotation: Vector3::new(0.0, 0.0, 0.0),
//            scale: Vector3::new(5.0, 2.0, 4.0),
//        });
//        model_components.set(&id2, ModelComponent {
//            mesh_name: "cube".to_owned(),
//            texture_name: "white".to_owned(),
//        });
//        let id3 = ecs.new_entity();
//        transform_components.set(&id3, TransformComponent {
//            position: Vector3::new(1.0, 0.0, 1.0),
//            rotation: Vector3::new(0.0, 0.0, 0.0),
//            scale: Vector3::new(1.0, 1.0, 1.0),
//        });
//        model_components.set(&id3, ModelComponent {
//            mesh_name: "cube".to_owned(),
//            texture_name: "bonjour".to_owned(),
//        });
//let id4 = ecs.new_entity();
//        transform_components.set(&id4, TransformComponent {
//            position: Vector3::new(1.0, 1.0, 1.0),
//            rotation: Vector3::new(0.0, 0.0, 0.0),
//            scale: Vector3::new(1.0, 1.0, 1.0),
//        });
//        model_components.set(&id4, ModelComponent {
//            mesh_name: "cube".to_owned(),
//            texture_name: "bonjour".to_owned(),
//        });
//
//
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

/// Macro to set up the component arrays in the ECS. It should be used with
macro_rules! register_components {
    { $([$name:ident, $component:ty],)+ } => {

        #[derive(Debug, Serialize, Deserialize)]
        struct Components {
            /// Size of the arrays. They all should be the same.
            current_size: usize,

            /// Arrays can be accessed with the get methods.
            $(
                pub $name: EntityArray<$component>,   
            )+
        }

        impl Components {
            pub fn new() -> Self {
                Components {
                    current_size: 0,
                    $(
                        $name: GenerationalIndexArray(Vec::new()),
                    )+
                }
            }

            /// We assume that this method is called by the ECS, so the index
            /// should be safe (meaning that either all the arrays contain
            /// the index, or the index is the next one when pushing elements
            /// to the arrays)...
            pub fn new_entity(&mut self, entity: &GenerationalIndex) {
                if entity.index() == self.current_size {
                    $(
                        self.$name.push(None);
                    )+
                } else if entity.index() < self.current_size {
                    $(
                        self.$name.empty(entity);
                    )+
                } else {
                    panic!("Tried to add an entity with index {}, but components arrays
                    only have elements up to {} entities", entity.index(), self.current_size);
                }

            }

            pub fn new_from_template(&mut self,
                                     entity: &GenerationalIndex,
                                     template: ComponentTemplate) {
                self.new_entity(entity);

                $(
                if template.$name.is_some() {
                    self.$name.set(entity, template.$name.unwrap());
                }
                )+
                
            }
        }

        #[derive(Debug, Serialize, Deserialize)]
        pub struct ComponentTemplate {
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(default)]
                pub $name: Option<$component>,
            )+
        }

        impl ComponentTemplate {
            pub fn new() -> Self {
                ComponentTemplate {
                    $(
                    $name: None,
                    )+
                }
            }

        }
    }
}

register_components!(
    [transforms, TransformComponent],
    [models, ModelComponent],
    );
