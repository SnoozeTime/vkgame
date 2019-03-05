use imgui::{Ui, im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2};
use crate::ecs::{
    ECS,
    Entity,
    new_component_popup,

    components::LightType,
};
use crate::ui::Gui;
use std::collections::HashMap;

pub struct Editor {
    pub selected_entity: Option<Entity>,
    pub hovered: bool,

    // For the component creation popup
    pub should_add_comp: bool,
    pub new_component_name: Option<String>,

    // For the components
    // dirty :D
    pub components_state: HashMap<String, String>,
}

impl Editor {

    pub fn new() -> Self {
        Editor {
            selected_entity: None,
            hovered: false,
            new_component_name: None,
            should_add_comp: false,
            components_state: HashMap::new(),
        }
    }
}

impl Editor {

    fn select_entity(&mut self, entity: Entity,
                     ecs: &ECS) {
        
        // reset component states.
        self.components_state.clear();
        if let Some(light_ref) = ecs.components.lights.get(&entity) {
            let v = match light_ref.light_type {
                LightType::Point => String::from("Point"),
                LightType::Directional => String::from("Directional"),
                LightType::Ambient => String::from("Ambient"),
            };

            self.components_state.insert("light.type".to_string(), v);
        }

        self.selected_entity = Some(entity);


    }


}

impl Gui for Editor {
    /// This is the function that will create the GUI!
    fn run_ui(&mut self,
              ui: &Ui,
              ecs: &mut ECS) -> bool {

        // Should be first. Reset the state
        self.hovered = false;

        ui.main_menu_bar(|| {
            ui.menu(im_str!("File")).build(|| {
                self.hovered = true;
                ui.menu_item(im_str!("Save"))
                    .shortcut(im_str!("CTRL+S"))
                    .build();
                ui.menu_item(im_str!("Load"))
                    .shortcut(im_str!("CTRL+L"))
                    .build();
            });

            ui.menu(im_str!("Edit")).build(|| {
                self.hovered = true;
                if ui.menu_item(im_str!("New entity"))
                    .build() {
                        let entity = ecs.new_entity();
                        self.select_entity(entity, ecs);
                    }
                if ui.menu_item(im_str!("New component"))
                    .enabled(self.selected_entity.is_some())
                        .build() {
                            new_component_popup(ui, self);
                        }
            });

        });

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

                        self.select_entity(*entity, ecs);
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

                ecs.components.draw_ui(&ui, self);
                if self.should_add_comp {
                    ecs.add_new_component_by_name(&self.selected_entity,
                                                  &self.new_component_name);
                    self.should_add_comp = false;
                    self.new_component_name = None;
                }
            });


        self.hovered = ui.want_capture_mouse();

        true
    }
}
