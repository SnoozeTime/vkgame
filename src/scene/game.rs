use cgmath::Vector3;
use imgui::Ui;
use log::debug;
use std::time::Duration;

use super::Scene;
use crate::camera::{Camera, CameraDirection, CameraInputHandler};
use crate::ecs::{
    components::TransformComponent,
    systems::{DummySystem, RenderingSystem},
    ECS,
};
use crate::event::{Event, GameEvent};
use crate::input::{Axis, Input, KeyType};
use crate::resource::Resources;
use crate::ui::Gui;

pub struct GameUi {}
impl Gui for GameUi {
    fn run_ui(&mut self, _ui: &Ui, _ecs: &mut ECS) -> bool {
        // maybe add debug console :D
        true
    }
}

pub struct GameScene {
    pub ecs: ECS,
    pub game_ui: GameUi,
    // All systems for this Scene.
    // dummy_system: DummySystem,
}

impl GameScene {
    pub fn new<'a>(render_system: &RenderingSystem<'a>) -> Self {
        let ecs = ECS::new();
        GameScene::from_ecs(ecs, render_system)
    }

    pub fn from_path<'a>(path: String, render_system: &RenderingSystem<'a>) -> Self {
        let ecs = ECS::load(path).unwrap();

        GameScene::from_ecs(ecs, render_system)
    }

    pub fn from_ecs<'a>(mut ecs: ECS, render_system: &RenderingSystem<'a>) -> Self {
        let transform = TransformComponent {
            position: Vector3::new(0.0, 1.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };

        let dimensions = render_system.dimensions();
        let aspect = (dimensions[0] as f32) / (dimensions[1] as f32);
        ecs.camera = Camera::new(transform, aspect, CameraInputHandler::fps_handler());

        GameScene {
            ecs,
            game_ui: GameUi {},
            //dummy_system: DummySystem::new(),
        }
    }
}

impl Scene for GameScene {
    fn update(&mut self, _dt: Duration) -> Option<Vec<Event>> {
        //self.dummy_system.do_dumb_thing(dt, &mut self.ecs);
        None
    }

    fn process_input(
        &mut self,
        input: Option<&Input>,
        _resources: Option<&Resources>,
        dt: Duration,
    ) -> Option<Vec<Event>> {
        let input = input.unwrap();
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

        let mut events: Option<Vec<Event>> = None;
        if input.get_key_down(KeyType::Escape) {
            if let None = events {
                events = Some(vec![Event::GameEvent(GameEvent::QuitGame)]);
            } else {
                events
                    .as_mut()
                    .unwrap()
                    .push(Event::GameEvent(GameEvent::QuitGame));
            }
        }

        events
    }

    fn get_parts_mut(&mut self) -> (&mut ECS, Option<&mut Gui>) {
        (&mut self.ecs, Some(&mut self.game_ui))
    }

    fn get_ecs(&self) -> &ECS {
        &self.ecs
    }
}
