use cgmath::{Point3, Vector3};
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



