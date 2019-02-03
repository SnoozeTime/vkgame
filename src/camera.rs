
use cgmath::{InnerSpace, Matrix4, Vector3, Rad, Angle, Point3};
use serde_derive::{Serialize, Deserialize};
use crate::ecs::components::TransformComponent;
use std::fmt;

pub struct CameraInputHandler {
    keyboard_handler: Box<FnMut(&mut CameraState, CameraDirection) -> ()>,
    mouse_handler: Box<FnMut(&mut CameraState, f64, f64) -> ()>,
}

impl fmt::Debug for CameraInputHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CameraInputHandler")
    }
}

impl CameraInputHandler {
    pub fn new<FK: 'static, FM: 'static>(keyboard_handler: FK,
                                         mouse_handler: FM) -> Self 
        where FK: FnMut(&mut CameraState, CameraDirection) -> (),
              FM: FnMut(&mut CameraState, f64, f64) -> () {
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

        let mut first_mouse = true;
        CameraInputHandler {
            keyboard_handler: Box::new(move |ref mut camera, direction| {

                match direction {
                    CameraDirection::Forward => camera.transform.position += 0.05*camera.front,
                    CameraDirection::Backward => camera.transform.position -= 0.05 * camera.front,
                    CameraDirection::Left => camera.transform.position -= 0.05 * camera.right,
                    CameraDirection::Right => camera.transform.position += 0.05 * camera.right,
                }


            }),
            mouse_handler: Box::new(move |ref mut camera, mouse_x, mouse_y| {
                if first_mouse {
                    camera.previous_x = mouse_x;
                    camera.previous_y = mouse_y;
                    first_mouse = false;
                }
                let x_offset = (camera.previous_x - mouse_x) as f32;
                let y_offset = (camera.previous_y - mouse_y) as f32;

                camera.previous_x = mouse_x;
                camera.previous_y = mouse_y;
                camera.pitch += 0.02 * y_offset;
                camera.yaw -= 0.02 * x_offset;

                if camera.pitch > 89.0 {
                    camera.pitch = 89.0;
                } else if camera.pitch < -89.0 {
                    camera.pitch = -89.0;
                }

                camera.update_vectors();

            }),
        }

    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Camera {
    state: CameraState,

    #[serde(skip)]
    #[serde(default = "CameraInputHandler::fps_handler")]
    input_handler: CameraInputHandler,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CameraState {

    // -----------------------------
    transform: TransformComponent,

    
    #[serde(with = "crate::ser::VectorDef")]
    front: Vector3<f32>,
    #[serde(with = "crate::ser::VectorDef")]
    right: Vector3<f32>,
    #[serde(with = "crate::ser::VectorDef")]
    up: Vector3<f32>,

    // Up of the woooorld
    #[serde(with = "crate::ser::VectorDef")]
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
        self.right = -self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraDirection {
    Forward,
    Backward,
    Right,
    Left,
}

impl Camera {
    pub fn new(transform: TransformComponent) -> Self {


        let front = Vector3::new(0.0, 0.0, -1.0);

        // Vulkan y-axis is downward
        let world_up = Vector3::new(0.0, -1.0, 0.0);

        let right = -front.cross(world_up).normalize();
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
        let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_2),
                                       1.0,
                                       0.01,
                                       100.0);
        let position = Point3::new(self.state.transform.position.x,
                                   self.state.transform.position.y,
                                   self.state.transform.position.z);
        let v = Matrix4::look_at(position, position + self.state.front, self.state.up);
        (v, proj)
    }

    pub fn process_keyboard(&mut self, direction: CameraDirection) {
        let handler = &mut self.input_handler.keyboard_handler;
        handler(&mut self.state, direction);
    }

    pub fn process_mouse(&mut self, mouse_x: f64, mouse_y: f64) {
        let handler = &mut self.input_handler.mouse_handler;
        handler(&mut self.state, mouse_x, mouse_y);
    }
}
