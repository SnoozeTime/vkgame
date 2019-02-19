use std::time::Duration;

use crate::resource::Resources;
use crate::input::Input;
use crate::ui::Gui;
use crate::ecs::ECS;
/// 
/// Scene should always have ECS. TODO how to enforce that?
pub trait Scene {
    fn update(&mut self, dt: Duration);
    fn process_input(&mut self, input: &Input, resources: &Resources, dt: Duration);
    fn get_parts_mut(&mut self) -> (&mut ECS, &mut Gui);
}

pub use editor::EditorScene;
pub use game::GameScene;

mod editor;
mod game;


