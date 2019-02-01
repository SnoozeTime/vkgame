use serde_derive::{Serialize, Deserialize};
use cgmath::Vector3;

use crate::ser::VectorDef;

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
