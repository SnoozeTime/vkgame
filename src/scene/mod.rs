use std::time::Duration;

use crate::resource::Resources;
use crate::input::Input;
use crate::ui::Gui;
use crate::ecs::ECS;
use crate::event::Event;

pub trait Scene {
    fn update(&mut self, dt: Duration) -> Option<Vec<Event>>;

    /// Will process input from window. 
    /// input and resources are optional because they will be only client-side.
    /// Server side will process terminal inputs maybe :)
    fn process_input(&mut self,
                     input: Option<&Input>,
                     resources: Option<&Resources>,
                     dt: Duration) -> Option<Vec<Event>>;

    fn get_parts_mut(&mut self) -> (&mut ECS, Option<&mut Gui>);
    fn get_ecs(&self) -> &ECS;
}

pub use editor::EditorScene;
pub use game::GameScene;
pub use netscene::NetworkScene;

mod editor;
mod game;
mod netscene;

pub struct SceneStack(Vec<Box<dyn Scene>>);

impl SceneStack {

    pub fn new() -> Self {
        SceneStack(Vec::new())
    }

    pub fn get_current(&mut self) -> Option<&mut Box<dyn Scene>> {
        let len = self.0.len();
        if len > 0 {
            self.0.get_mut(len - 1)
        } else {
            None
        }
    }

    pub fn pop(&mut self) -> Option<Box<dyn Scene>> {
        self.0.pop() 
    }

    pub fn push<T>(&mut self, scene: T) 
        where T: Scene + 'static {
        self.0.push(Box::new(scene));
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
