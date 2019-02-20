use imgui::{Ui, im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2};
use crate::ecs::{
    ECS,
    Entity,
};
use crate::ui::Gui;

pub struct Editor {
    pub selected_entity: Option<Entity>,
    pub hovered: bool,
}

impl Editor {

    pub fn new() -> Self {
        Editor {
            selected_entity: None,
            hovered: false,
        }
    }
}

impl Gui for Editor {

    /// This is the function that will create the GUI!
    fn run_ui(&mut self,
              ui: &Ui,
              ecs: &mut ECS) -> bool {

        self.hovered = false;
        ui.window(im_str!("Scene"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                let live_entities = ecs.nb_entities();
                for (i, entity) in live_entities.iter().enumerate() {

                    let selected = {
                        if let Some(selected) = self.selected_entity {
                            selected.index() == i
                        } else { false }
                    };
                    if ui.selectable(
                        im_str!("Entity {}", i),
                        selected,
                        ImGuiSelectableFlags::empty(),
                        ImVec2::new(0.0, 0.0)) {

                        self.selected_entity = Some(*entity);
                    }
                }
                if ui.is_window_hovered() || ui.is_window_focused() {
                    self.hovered = true;
                }
            });
        ui.window(im_str!("Components"))
            .size((200.0, 500.0), ImGuiCond::FirstUseEver)
            .build(|| {

                if ui.is_window_hovered() || ui.is_window_focused() {
                    self.hovered = true;
                }

                ecs.components.draw_ui(&ui, &self);
            });


        true
    }
}
