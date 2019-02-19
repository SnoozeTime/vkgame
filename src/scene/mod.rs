use std::time::Duration;

use crate::resource::Resources;
use crate::input::Input;
use crate::ui::Gui;
use crate::ecs::ECS;
use crate::event::Event;
/// 
/// Scene should always have ECS. TODO how to enforce that?
/// TODO Vec of events? Can't it be a slice?
pub trait Scene {
    fn update(&mut self, dt: Duration) -> Option<Vec<Event>>;
    fn process_input(&mut self, input: &Input, resources: &Resources, dt: Duration) -> Option<Vec<Event>>;
    fn get_parts_mut(&mut self) -> (&mut ECS, &mut Gui);
}

pub use editor::EditorScene;
pub use game::GameScene;

mod editor;
mod game;


