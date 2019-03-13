use std::default::Default;
use cgmath::SquareMatrix;
use cgmath::{InnerSpace, Matrix4, Vector3, Rad, Angle, Point3};
use serde_derive::{Serialize, Deserialize};
use crate::ecs::components::TransformComponent;
use std::fmt;
use std::time::Duration;
use crate::time::dt_as_secs;

pub struct CameraInputHandler {
    keyboard_handler: Box<FnMut(&mut CameraState, Duration, CameraDirection) -> ()>,
    mouse_handler: Box<FnMut(&mut CameraState, Duration, f64, f64) -> ()>,
}

impl fmt::Debug for CameraInputHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CameraInputHandler")
    }
}

impl CameraInputHandler {
    pub fn new<FK: 'static, FM: 'static>(keyboard_handler: FK,
                                         mouse_handler: FM) -> Self 
        where FK: FnMut(&mut CameraState, Duration, CameraDirection) -> (),
              FM: FnMut(&mut CameraState, Duration, f64, f64) -> () {
                  CameraInputHandler {
                      keyboard_handler: Box::new(keyboard_handler),
                      mouse_handler: Box::new(mouse_handler),
                  }
              }

    pub fn noop_handler() -> Self {
        CameraInputHandler {
            keyboard_handler: Box::new(|ref mut _camera, _dt, _direction| {}),
            mouse_handler: Box::new(|ref mut _camera, _dt, _mouse_x, _mouse_y| {}),
        }
    }

    pub fn free_handler() -> Self {

        CameraInputHandler {
            keyboard_handler: Box::new(move |ref mut camera, dt, direction| {

                match direction {
                    CameraDirection::Forward => camera.transform.position += camera.speed * dt_as_secs(dt) as f32 * camera.front,
                    CameraDirection::Backward => camera.transform.position -= camera.speed * dt_as_secs(dt) as f32 * camera.front,
                    CameraDirection::Left => camera.transform.position -= camera.speed * dt_as_secs(dt) as f32 * camera.right,
                    CameraDirection::Right => camera.transform.position += camera.speed * dt_as_secs(dt) as f32 * camera.right,
                }


            }),
            mouse_handler: Box::new(move |ref mut camera, dt, mouse_x, mouse_y| {
                let dt = dt_as_secs(dt) as f32;
                let mouse_x = mouse_x as f32;
                let mouse_y = mouse_y as f32;
                let x_offset = mouse_x * dt;
                let y_offset = mouse_y * dt;

                camera.pitch -= 0.24 * y_offset;
                camera.yaw += 0.24 * x_offset;

                if camera.pitch > 89.0 {
                    camera.pitch = 89.0;
                } else if camera.pitch < -89.0 {
                    camera.pitch = -89.0;
                }

                camera.update_vectors();
            }),
        }

    }

    pub fn fps_handler() -> Self {
        let up = Vector3::new(0.0, 1.0, 0.0);

        CameraInputHandler {
            keyboard_handler: Box::new(move |ref mut camera, dt, direction| {

                // projection on plane.
                let proj_front = camera.front - (camera.front.dot(up)) * up;
                let proj_right = camera.right - (camera.right.dot(up)) * up;
                match direction {
                    CameraDirection::Forward => camera.transform.position += camera.speed * dt_as_secs(dt) as f32 *proj_front,
                    CameraDirection::Backward => camera.transform.position -= camera.speed * dt_as_secs(dt) as f32 * proj_front,
                    CameraDirection::Left => camera.transform.position -= camera.speed * dt_as_secs(dt) as f32 * proj_right,
                    CameraDirection::Right => camera.transform.position += camera.speed * dt_as_secs(dt) as f32 * proj_right,
                }


            }),
            mouse_handler: Box::new(move |ref mut camera, dt, mouse_x, mouse_y| {
                let dt = dt_as_secs(dt) as f32;
                let mouse_x = mouse_x as f32;
                let mouse_y = mouse_y as f32;
                let x_offset = mouse_x * dt;
                let y_offset = mouse_y * dt;

                camera.pitch -= 0.50 * y_offset;
                camera.yaw += 0.50 * x_offset;

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

    aspect: f32,

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

    speed: f32,
}

impl CameraState {

    fn update_vectors(&mut self) {
        let front_z = -Rad(self.yaw).cos() * Rad(self.pitch).cos(); 
        let front_y = Rad(self.pitch).sin(); 
        let front_x = Rad(self.yaw).sin() * Rad(self.pitch).cos(); 
        self.front = Vector3::new(front_x, front_y, front_z).normalize();
        self.right = self.front.cross(self.world_up).normalize();
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

impl Default for Camera {

    fn default() -> Self {


        let front = Vector3::new(0.0, 0.0, -1.0);
        let world_up = Vector3::new(0.0, 1.0, 0.0);

        let right = front.cross(world_up).normalize();
        let up = right.cross(front).normalize();

        let pitch = 0.0;
        let yaw = 0.0;

        let transform = TransformComponent {
            position: Vector3::new(0.0, 1.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };

        let aspect = 1.0;
        let speed = 10.0; // unit/secs

        let state = CameraState {
            transform,
            front,
            up,
            right,
            world_up,
            yaw,
            pitch,
            aspect,
            speed,
        };

        let input_handler = CameraInputHandler::fps_handler();
        Camera {
            state,
            input_handler,
        }
    }
}

impl Camera {
    pub fn new(transform: TransformComponent, aspect: f32, input_handler: CameraInputHandler) -> Self {

        let front = Vector3::new(0.0, 0.0, -1.0);
        let world_up = Vector3::new(0.0, 1.0, 0.0);

        let right = front.cross(world_up).normalize();
        let up = right.cross(front).normalize();

        let pitch = 0.0;
        let yaw = 0.0;
        let speed = 10.0;

        let state = CameraState {
            transform,
            front,
            up,
            right,
            world_up,
            yaw,
            pitch,
            aspect,
            speed,
        };

        Camera {
            state,
            input_handler,
        }
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.state.aspect = aspect;
    }

    pub fn get_vp(&self) -> (Matrix4<f32>, Matrix4<f32>) {
        let proj = cgmath::perspective(Rad(0.6*std::f32::consts::FRAC_PI_2),
        self.state.aspect,
        0.01,
        100.0);
        let position = Point3::new(self.state.transform.position.x,
                                   self.state.transform.position.y,
                                   self.state.transform.position.z);
        let v = Matrix4::look_at(position, position + self.state.front, self.state.up);

        // fix projection for vulkan.
        // See details here: https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
        let mut the_fix = Matrix4::identity();
        the_fix[1][1] = -1.0;
        the_fix[2][3] = 0.5;
        the_fix[2][2] = 0.5;
        (v, the_fix*proj)
    }

    pub fn process_keyboard(&mut self, dt: Duration, direction: CameraDirection) {
        let handler = &mut self.input_handler.keyboard_handler;
        handler(&mut self.state, dt, direction);
    }

    pub fn process_mouse(&mut self, dt: Duration, mouse_x: f64, mouse_y: f64) {
        let handler = &mut self.input_handler.mouse_handler;
        handler(&mut self.state, dt, mouse_x, mouse_y);
    }

    pub fn transform(&self) -> &TransformComponent {
        &self.state.transform
    }
}
