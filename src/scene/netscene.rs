/// Just store the ECS and systems. 

use crate::ecs::ECS;
use super::Scene;
use crate::event::Event;
use crate::ui::Gui;
use std::time::Duration;
use crate::input::Input;
use crate::resource::Resources;
use crate::net::NetworkSystem;

pub struct NetworkScene {

    // All the objects.
    ecs: ECS,

    // My nice systems
    network: NetworkSystem,
}

impl NetworkScene {
    pub fn new(port: usize, max_clients: usize) -> Self {

        // can crash if problem with network. Don't worry, that is life.
        let network = NetworkSystem::new(port, max_clients);

        NetworkScene {
            network,
            ecs: ECS::new()
        }
    }
}

impl Scene for NetworkScene {
    
    
    fn update(&mut self, dt: Duration) -> Option<Vec<Event>> {

        None
    }

    fn process_input(&mut self,
                     input: Option<&Input>,
                     resources: Option<&Resources>,
                     dt: Duration) -> Option<Vec<Event>> {

        None
    }

    fn get_parts_mut(&mut self) -> (&mut ECS, Option<&mut Gui>) {
        (&mut self.ecs, None)
    }
    
    
    fn get_ecs(&self) -> &ECS {
        &self.ecs
    }
}


