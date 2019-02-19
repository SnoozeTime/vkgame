use std::time::Duration;

use crate::ecs::{
    ECS,
    systems::{RenderingSystem},
};
use crate::editor::Editor;
use crate::camera::{CameraDirection};
use crate::input::{KeyType, Input, Axis, MouseButton};
use crate::renderer::pick::Object3DPicker;
use crate::resource::Resources;

pub trait Scene {
    fn update(&mut self, dt: Duration);
    fn process_input(&mut self, input: &Input, resources: &Resources, dt: Duration);
}

pub struct EditorScene {
    pub ecs: ECS,
    pub editor: Editor,

    object_picker: Object3DPicker, 
}

impl EditorScene {
    
    pub fn new<'a>(render_system: &RenderingSystem<'a>) -> Self {
        EditorScene {
            ecs: ECS::dummy_ecs(),
            editor: Editor::new(),
            object_picker: Object3DPicker::new(
                render_system.get_device(),
                render_system.get_queue(),
                render_system.get_surface(),
                render_system.dimensions(),
                ),
        }
    }

    pub fn from_path<'a>(path: String, render_system: &RenderingSystem<'a>) -> Self {
        EditorScene {
            ecs: ECS::load(path).unwrap(),
            editor: Editor::new(),
            object_picker: Object3DPicker::new(
                render_system.get_device(),
                render_system.get_queue(),
                render_system.get_surface(),
                render_system.dimensions(),
                ),
        }
    }
}

impl Scene for EditorScene {

    fn update(&mut self, _dt: Duration) {}

    fn process_input(&mut self,
                     input: &Input,
                     resources: &Resources,
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

        if input.get_mouse_clicked(MouseButton::Left) && !self.editor.hovered {
            self.editor.selected_entity = self.object_picker.pick_object(input.mouse_pos[0],
                                                                    input.mouse_pos[1],
                                                                    &self.ecs,
                                                                    &resources.models);
        }

    }
}

