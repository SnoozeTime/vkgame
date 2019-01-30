
use cgmath::{InnerSpace, Matrix4, Vector3, Rad, Angle, Point3};
use crate::gameobject::Transform;
use serde_derive::{Serialize, Deserialize};

pub struct CameraInputHandler {
    keyboard_handler: Box<Fn(&mut CameraState, CameraDirection) -> ()>,
    mouse_handler: Box<Fn(&mut CameraState, f64, f64) -> ()>,
}

impl CameraInputHandler {
    pub fn new<FK: 'static, FM: 'static>(keyboard_handler: FK,
                                         mouse_handler: FM) -> Self 
        where FK: Fn(&mut CameraState, CameraDirection) -> (),
              FM: Fn(&mut CameraState, f64, f64) -> () {
                  CameraInputHandler {
                      keyboard_handler: Box::new(keyboard_handler),
                      mouse_handler: Box::new(mouse_handler),
                  }
              }

    pub fn noop_handler() -> Self {
        CameraInputHandler {
            keyboard_handler: Box::new(|ref mut _camera, _direction| {}),
            mouse_handler: Box::new(|ref mut _camera, _mouse_x, _mouse_y| {}),
        }
    }

    pub fn fps_handler() -> Self {
        CameraInputHandler {
            keyboard_handler: Box::new(|ref mut camera, direction| {

                match direction {
                    CameraDirection::Forward => camera.transform.position += 0.05*camera.front,
                    CameraDirection::Backward => camera.transform.position -= 0.05 * camera.front,
                    CameraDirection::Left => camera.transform.position -= 0.05 * camera.right,
                    CameraDirection::Right => camera.transform.position += 0.05 * camera.right,
                }


            }),
            mouse_handler: Box::new(|ref mut camera, mouse_x, mouse_y| {
                let x_offset = (camera.previous_x - mouse_x) as f32;
                let y_offset = (camera.previous_y - mouse_y) as f32;

                camera.previous_x = mouse_x;
                camera.previous_y = mouse_y;
                camera.pitch += 0.02 * y_offset;
                camera.yaw -= 0.02 * x_offset;
                camera.update_vectors();

            }),
        }

    }
}

pub struct Camera {
    state: CameraState,
    input_handler: CameraInputHandler,
}

impl Camera {

}

pub struct CameraState {

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

impl CameraState {

    fn update_vectors(&mut self) {
        let front_x = Rad(self.yaw).cos() * Rad(self.pitch).cos(); 
        let front_y = Rad(self.pitch).sin(); 
        let front_z = Rad(self.yaw).sin() * Rad(self.pitch).cos(); 

        self.front = Vector3::new(front_x, front_y, front_z).normalize();
        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

}

pub enum CameraDirection {
    Forward,
    Backward,
    Right,
    Left,
}

impl Camera {
    pub fn new(transform: Transform) -> Self {


        let front = Vector3::new(0.0, 0.0, -1.0);
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let right = front.cross(world_up).normalize();
        let up = right.cross(front).normalize();

        let pitch = 0.0;
        let yaw = 0.0;
        let previous_x = 0.0;
        let previous_y = 0.0;

        let state = CameraState {
            transform,
            front,
            up,
            right,
            world_up,
            yaw,
            pitch,
            previous_x,
            previous_y,
        };
        
        let input_handler = CameraInputHandler::fps_handler();
        Camera {
            state,
            input_handler,
        }
    }

    pub fn get_vp(&self) -> (Matrix4<f32>, Matrix4<f32>) {
        let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2), 1.0, 0.01, 100.0);
        let position = Point3::new(self.state.transform.position.x,
                                   self.state.transform.position.y,
                                   self.state.transform.position.z);
        let v = Matrix4::look_at(position, position + self.state.front, self.state.up);
        (v, proj)
    }

    pub fn process_keyboard(&mut self, direction: CameraDirection) {
        let handler = &self.input_handler.keyboard_handler;
        handler(&mut self.state, direction);
    }

    pub fn process_mouse(&mut self, mouse_x: f64, mouse_y: f64) {
        let handler = &self.input_handler.mouse_handler;
        handler(&mut self.state, mouse_x, mouse_y);
    }
}

// -----------------------------------------------------
pub struct CameraBuilder {

}
