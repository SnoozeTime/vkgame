
use cgmath::{InnerSpace, Matrix4, Vector3, Rad, Angle};
use crate::gameobject::Transform;

#[derive(Debug, Clone)]
pub struct Camera {

    // -----------------------------
    transform: Transform,

    front: Vector3<f32>,
    right: Vector3<f32>,
    up: Vector3<f32>,

    // Up of the woooorld
    world_up: Vector3<f32>,

    pitch: f32,
    yaw: f32,

    // ----------------------------
    previous_x: f64,
    previous_y: f64,
}

pub enum CameraDirection {
    Forward,
    Backward,
    Right,
    Left,
}

impl Camera {

    pub fn new(transform: Transform) -> Camera {
        
        let front = Vector3::new(0.0, 0.0, -1.0);
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let right = front.cross(world_up).normalize();
        let up = right.cross(front).normalize();

        let pitch = 0.0;
        let yaw = 0.0;
        let previous_x = 0.0;
        let previous_y = 0.0;
        
        Camera {
            transform,
            front,
            up,
            right,
            world_up,
            yaw,
            pitch,
            previous_x,
            previous_y,
        }

    }

    pub fn look_at(&self) -> Matrix4<f32> {
        Matrix4::look_at(self.transform.position, self.transform.position + self.front, self.up)  
    }

    pub fn process_keyboard(&mut self, direction: CameraDirection) {
        match direction {
            CameraDirection::Forward => self.transform.position += 0.05*self.front,
            CameraDirection::Backward => self.transform.position -= 0.05 * self.front,
            CameraDirection::Left => self.transform.position -= 0.05 * self.right,
            CameraDirection::Right => self.transform.position += 0.05 * self.right,
        }
    }

    pub fn process_mouse(&mut self, mouse_x: f64, mouse_y: f64) {
        let x_offset = (self.previous_x - mouse_x) as f32;
        let y_offset = (self.previous_y - mouse_y) as f32;

        self.previous_x = mouse_x;
        self.previous_y = mouse_y;
        self.pitch -= 0.02 * y_offset;
        self.yaw -= 0.02 * x_offset;
        self.update_vectors();
    }

    fn update_vectors(&mut self) {
       let front_x = Rad(self.yaw).cos() * Rad(self.pitch).cos(); 
       let front_y = Rad(self.pitch).sin(); 
       let front_z = Rad(self.yaw).sin() * Rad(self.pitch).cos(); 

       self.front = Vector3::new(front_x, front_y, front_z).normalize();
       self.right = self.front.cross(self.world_up).normalize();
       self.up = self.right.cross(self.front).normalize();
    }
}


