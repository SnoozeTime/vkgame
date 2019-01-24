use cgmath::{Matrix3, Matrix4, Point3, Vector3, Rad};

pub mod camera;
#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Point3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Point3<f32>,
}


