use super::Scene;
use crate::ecs::systems::PlayerSystem;
/// Just store the ECS and systems.
use crate::ecs::ECS;
use crate::event::Event;
use crate::input::Input;
use crate::net::NetworkSystem;
use crate::resource::Resources;
use crate::ui::Gui;
use log::debug;
use std::time::Duration;

pub struct NetworkScene {
    // All the objects.
    ecs: ECS,

    // My nice systems
    network: NetworkSystem,
    player_system: PlayerSystem,
}

impl NetworkScene {
    pub fn new(port: usize, max_clients: usize) -> Self {
        // can crash if problem with network. Don't worry, that is life.
        let network = NetworkSystem::new(port, max_clients);

        NetworkScene {
            network,
            ecs: ECS::new(),
            player_system: PlayerSystem::new(),
        }
    }

    pub fn from_file(port: usize, max_clients: usize, filename: String) -> Self {
        let network = NetworkSystem::new(port, max_clients);
        let ecs = ECS::load(filename).unwrap();
        let player_system = PlayerSystem::new();
        NetworkScene {
            network,
            ecs,
            player_system,
        }
    }
}

impl Scene for NetworkScene {
    fn update(&mut self, dt: Duration) -> Option<Vec<Event>> {
        // Get the latest event from the clients.
        let events = self.network.poll_events(&mut self.ecs);
        self.player_system
            .handle_network_events(&mut self.ecs, &events);

        // All the systems.
        self.player_system.update(dt, &mut self.ecs);

        // Finish by sending latest state.
        self.network.send_state(&mut self.ecs);

        None
    }

    fn process_input(
        &mut self,
        input: Option<&Input>,
        resources: Option<&Resources>,
        dt: Duration,
    ) -> Option<Vec<Event>> {
        None
    }

    fn get_parts_mut(&mut self) -> (&mut ECS, Option<&mut Gui>) {
        (&mut self.ecs, None)
    }

    fn get_ecs(&self) -> &ECS {
        &self.ecs
    }
}
