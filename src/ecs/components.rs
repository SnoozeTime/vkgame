use serde_derive::{Serialize, Deserialize};
use cgmath::Vector3;

use crate::ser::VectorDef;

enum ComponentType {
    Transform,
    Model,
}

/// This is a component that is going to be rendered
/// by the render system.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelComponent {
    // name of the mesh and the texture, These need to be
    // loaded at the start of the scene.
    pub mesh_name: String,
    pub texture_name: String,
}

/// Position of the game object. No position = no rendering.
#[derive(Debug, Serialize, Deserialize)]
pub struct TransformComponent {
    #[serde(with = "VectorDef")]
    pub position: Vector3<f32>,

    #[serde(with = "VectorDef")]
    pub rotation: Vector3<f32>,

    #[serde(with = "VectorDef")]
    pub scale: Vector3<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DummyComponent {
    pub speed: f32,
}

// Emit light! Right now, only one is supported.
// An entity with a light component will need a transform.
#[derive(Debug, Serialize, Deserialize)]
pub struct LightComponent {
    
    // Should be between 0 and 1.0
    pub color: [f32; 3],

    // TODO light type.

}
