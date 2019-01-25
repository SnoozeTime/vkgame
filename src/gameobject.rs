use cgmath::{Matrix3, Matrix4, Point3, Vector3, Rad};
use crate::camera::Camera;

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Point3<f32>,
}

pub struct MeshComponent {
    pub mesh_name: Option<String>,
    pub texture_name: Option<String>,
}

pub struct Scene {
    pub transforms: Transform,
    pub mesh_components: MeshComponent,
    pub camera: Camera,
}



