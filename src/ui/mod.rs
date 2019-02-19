use imgui::Ui;
use crate::ecs::ECS;

pub trait Gui {
    fn run_ui(&mut self, ui: &Ui, ecs: &mut ECS) -> bool;
}
