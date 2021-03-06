use cgmath::Vector3;
use imgui::Ui;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::time::Duration;

use super::Scene;
use crate::camera::{Camera, CameraDirection, CameraInputHandler};
use crate::ecs::{
    components::TransformComponent,
    systems::{DummySystem, RenderingSystem},
    ECS,
};
use crate::event::Event;
use crate::input::{Axis, Input, KeyType};
use crate::resource::Resources;
use crate::ui::Gui;

use crate::net::ClientSystem;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum ClientCommand {
    Move(CameraDirection),
    LookAt([f32; 3]),
}

pub struct GameUi {}
impl Gui for GameUi {
    fn run_ui(&mut self, _ui: &Ui, _ecs: &mut ECS) -> bool {
        // maybe add debug console :D
        true
    }
}

pub struct ClientScene {
    pub ecs: ECS,
    pub game_ui: GameUi,

    // All systems for this Scene.
    // dummy_system: DummySystem,
    backend: ClientSystem,
    commands: Vec<ClientCommand>,
}

impl ClientScene {
    pub fn new<'a>(server_addr: &str, render_system: &RenderingSystem<'a>) -> Self {
        let mut ecs = ECS::new();
        let transform = TransformComponent {
            position: Vector3::new(0.0, 1.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        };

        let dimensions = render_system.dimensions();
        let aspect = (dimensions[0] as f32) / (dimensions[1] as f32);
        ecs.camera = Camera::new(transform, aspect, CameraInputHandler::fps_handler());

        let backend = ClientSystem::connect(server_addr.parse().unwrap()).unwrap();
        let commands = Vec::with_capacity(10);

        ClientScene {
            ecs,
            game_ui: GameUi {},
            backend,
            commands,
        }
    }
}

impl Scene for ClientScene {
    fn update(&mut self, _dt: Duration) -> Option<Vec<Event>> {
        //self.dummy_system.do_dumb_thing(dt, &mut self.ecs);
        self.backend.poll_events(&mut self.ecs);
        None
    }

    fn process_input(
        &mut self,
        input: Option<&Input>,
        _resources: Option<&Resources>,
        dt: Duration,
    ) -> Option<Vec<Event>> {
        let input = input.unwrap();

        self.commands.clear();
        if input.get_key(KeyType::Up) {
            self.commands
                .push(ClientCommand::Move(CameraDirection::Forward));
        }

        if input.get_key(KeyType::Down) {
            self.commands
                .push(ClientCommand::Move(CameraDirection::Backward));
        }

        if input.get_key(KeyType::Left) {
            self.commands
                .push(ClientCommand::Move(CameraDirection::Left));
            //self.ecs.camera.process_keyboard(dt, CameraDirection::Left);
        }

        if input.get_key(KeyType::Right) {
            self.commands
                .push(ClientCommand::Move(CameraDirection::Right));
        }

        let (h_axis, v_axis) = (
            input.get_axis(Axis::Horizontal),
            input.get_axis(Axis::Vertical),
        );
        if h_axis != 0.0 || v_axis != 0.0 {
            self.ecs.camera.process_mouse(dt, h_axis, v_axis);
            self.commands
                .push(ClientCommand::LookAt(self.ecs.camera.state.front.into()));
        }

        self.backend.send_commands(&self.commands);
        None
    }

    fn get_parts_mut(&mut self) -> (&mut ECS, Option<&mut Gui>) {
        (&mut self.ecs, Some(&mut self.game_ui))
    }

    fn get_ecs(&self) -> &ECS {
        &self.ecs
    }
}
