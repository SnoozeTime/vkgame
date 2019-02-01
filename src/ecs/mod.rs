use serde_derive::{Serialize, Deserialize};

pub mod components;
pub mod systems;
use self::components::{TransformComponent, ModelComponent};
use crate::camera::Camera;

// TODO change to generational ID.
type EntityId = usize;
type EntityArray<T> = Vec<Option<T>>;


#[derive(Serialize, Deserialize, Debug)]
pub struct ECS {

    // Allllll my components.
    transform_components: EntityArray<TransformComponent>,
    model_components: EntityArray<ModelComponent>,


    // For now, static camera. TODO add it as a component.
    camera: Camera,
}
