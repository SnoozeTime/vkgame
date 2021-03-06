use cgmath::Vector3;
use std::path::Path;
use std::time::Duration;

use super::Scene;
use crate::camera::{Camera, CameraDirection, CameraInputHandler};
use crate::ecs::{components::TransformComponent, systems::RenderingSystem, ECS};
use crate::editor::Editor;
use crate::event::{EditorEvent, Event};
use crate::input::{Axis, Input, KeyType, MouseButton};
use crate::renderer::pick::Object3DPicker;
use crate::resource::Resources;
use crate::ui::Gui;

pub struct EditorScene {
    pub ecs: ECS,
    pub editor: Editor,

    object_picker: Object3DPicker,
}

impl EditorScene {
    pub fn new(render_system: &RenderingSystem, resources: &Resources) -> Self {
        let ecs = ECS::dummy_ecs();
        EditorScene::from_ecs(ecs, render_system, resources)
    }

    pub fn from_path(path: String, render_system: &RenderingSystem, resources: &Resources) -> Self {
        let ecs = ECS::load(&path).unwrap();
        let mut scene = EditorScene::from_ecs(ecs, render_system, resources);
        scene.editor.load(&mut scene.ecs, path);
        scene
    }

    fn from_ecs<'a>(
        mut ecs: ECS,
        render_system: &RenderingSystem<'a>,
        resources: &Resources,
    ) -> Self {
        let transform = TransformComponent {
            position: Vector3::new(0.0, 1.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };

        let dimensions = render_system.dimensions();
        let aspect = (dimensions[0] as f32) / (dimensions[1] as f32);
        ecs.camera = Camera::new(transform, aspect, CameraInputHandler::free_handler());

        EditorScene {
            ecs,
            editor: Editor::new(resources),
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
    fn update(&mut self, _dt: Duration) -> Option<Vec<Event>> {
        None
    }

    fn process_input(
        &mut self,
        input: Option<&Input>,
        resources: Option<&Resources>,
        dt: Duration,
    ) -> Option<Vec<Event>> {
        let input = input.unwrap();
        let resources = resources.unwrap();

        let mut events: Option<Vec<Event>> = None;
        // HANDLE CAMERA.
        if input.modifiers.ctrl {
            if input.get_key(KeyType::Up) {
                self.ecs
                    .camera
                    .process_keyboard(dt, CameraDirection::Forward);
            }

            if input.get_key(KeyType::Down) {
                self.ecs
                    .camera
                    .process_keyboard(dt, CameraDirection::Backward);
            }

            if input.get_key(KeyType::Left) {
                self.ecs.camera.process_keyboard(dt, CameraDirection::Left);
            }

            if input.get_key(KeyType::Right) {
                self.ecs.camera.process_keyboard(dt, CameraDirection::Right);
            }

            let (h_axis, v_axis) = (
                input.get_axis(Axis::Horizontal),
                input.get_axis(Axis::Vertical),
            );
            if h_axis != 0.0 || v_axis != 0.0 {
                self.ecs.camera.process_mouse(dt, h_axis, v_axis);
            }
        }

        if input.get_mouse_clicked(MouseButton::Left) && !self.editor.hovered {
            self.editor.selected_entity = self.object_picker.pick_object(
                input.mouse_pos[0],
                input.mouse_pos[1],
                &self.ecs,
                &resources.models,
            );
        }

        if input.get_key_down(KeyType::Space) {
            if let None = events {
                events = Some(vec![Event::EditorEvent(EditorEvent::PlayGame)]);
            } else {
                events
                    .as_mut()
                    .unwrap()
                    .push(Event::EditorEvent(EditorEvent::PlayGame));
            }
        }
        if input.get_key_down(KeyType::Escape) {
            self.editor.request_quit();
        }

        if input.get_key_down(KeyType::NextScene) {
            self.editor.request_load_next();
        }
        if input.get_key_down(KeyType::PreviousScene) {
            self.editor.request_load_previous();
        }

        // Pending events from the editor. Maybe not best way to do this but hey!
        if let Some(ev) = self.editor.event_to_process.take() {
            match ev {
                Event::EditorEvent(EditorEvent::LoadNext) => {
                    self.editor.next_scene(&mut self.ecs);
                }

                Event::EditorEvent(EditorEvent::LoadPrevious) => {
                    self.editor.previous_scene(&mut self.ecs);
                }

                _ => {
                    if let None = events {
                        events = Some(vec![ev]);
                    } else {
                        events.as_mut().unwrap().push(ev);
                    }
                }
            }
        }

        events
    }

    fn get_parts_mut(&mut self) -> (&mut ECS, Option<&mut Gui>) {
        (&mut self.ecs, Some(&mut self.editor))
    }

    fn get_ecs(&self) -> &ECS {
        &self.ecs
    }
}
