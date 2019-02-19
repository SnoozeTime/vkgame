use std::time::Duration;

use crate::ecs::{
    ECS,
    systems::{RenderingSystem},
};
use crate::editor::Editor;
use crate::camera::{CameraDirection};
use crate::input::{KeyType, Input, Axis, MouseButton};

trait Scene {
    fn update(&mut self, dt: Duration);
    fn process_input(&mut self, input: &Input, dt: Duration);
}

pub struct EditorScene {
    ecs: ECS,
    editor: Editor,
}

impl EditorScene {
    
    pub fn new() -> Self {
        EditorScene {
            ecs: ECS::dummy_ecs(),
            editor: Editor::new(),
        }
    }

    pub fn from_path(path: String) -> Self {
        EditorScene {
            ecs: ECS::load(path).unwrap(),
            editor: Editor::new(),
        }
    }
}

impl Scene for EditorScene {

    fn update(&mut self, _dt: Duration) {}

    fn process_input(&mut self,
                     input: &Input,
                     dt: Duration) {

         // HANDLE CAMERA.
        if input.modifiers.ctrl {
            if input.get_key(KeyType::Up) {
                self.ecs.camera.process_keyboard(dt,
                                            CameraDirection::Forward);
            }

            if input.get_key(KeyType::Down) {
                self.ecs.camera.process_keyboard(dt,
                                            CameraDirection::Backward);
            }

            if input.get_key(KeyType::Left) {
                self.ecs.camera.process_keyboard(dt,
                                            CameraDirection::Left);
            }

            if input.get_key(KeyType::Right) {
                self.ecs.camera.process_keyboard(dt,
                                            CameraDirection::Right);
            }

            let (h_axis, v_axis) = (input.get_axis(Axis::Horizontal),
            input.get_axis(Axis::Vertical));
            if h_axis != 0.0 || v_axis != 0.0 {
                self.ecs.camera.process_mouse(dt,
                                              h_axis,
                                              v_axis);
            }
        }

//        if input.get_mouse_clicked(MouseButton::Left) && !self.editor.hovered {
//            self.editor.selected_entity = self.render_system.pick_object(input.mouse_pos[0],
//                                                                    input.mouse_pos[1],
//                                                                    &self.ecs);
//        }

    }
}

