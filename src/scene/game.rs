use std::time::Duration;
use cgmath::Vector3;
use imgui::Ui;

use crate::ecs::{
    ECS,
    systems::{RenderingSystem, DummySystem},
};
use crate::editor::Editor;
use crate::camera::{CameraDirection, Camera, CameraInputHandler};
use crate::input::{KeyType, Input, Axis, MouseButton};
use crate::renderer::pick::Object3DPicker;
use crate::resource::Resources;
use crate::ui::Gui;
use super::Scene;

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
    dummy_system: DummySystem,
}

impl GameScene {
    pub fn new<'a>(render_system: &RenderingSystem<'a>) -> Self {
        let mut ecs = ECS::dummy_ecs();
        let dimensions = render_system.dimensions();
        let aspect = (dimensions[0] as f32) / (dimensions[1] as f32);
        ecs.camera.set_aspect(aspect);
        GameScene {
            ecs,
            game_ui: GameUi{},
            dummy_system: DummySystem::new(),
        }
    }
}


impl Scene for GameScene {

    fn update(&mut self, dt: Duration) {
        self.dummy_system.do_dumb_thing(dt, &mut self.ecs);
    }

    fn process_input(&mut self,
                     input: &Input,
                     resources: &Resources,
                     dt: Duration) {

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

    fn get_parts_mut(&mut self) -> (&mut ECS, &mut Gui) {
        (&mut self.ecs, &mut self.game_ui)
    }
}

