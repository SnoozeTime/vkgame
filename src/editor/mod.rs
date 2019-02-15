use imgui::{FontGlyphRange, ImFontConfig, ImGui, Ui, im_str, ImGuiCond, ImDrawVert, ImGuiSelectableFlags, ImVec2};
use crate::ecs::ECS;

pub struct Editor {
    selected_entity: Option<usize>,
}

impl Editor {

    pub fn new() -> Self {
        Editor {
            selected_entity: None,
        }
    }


    /// This is the function that will create the GUI!
    pub fn run_ui(&mut self,
                  ui: &Ui,
                  ecs: &mut ECS) -> bool {

        ui.window(im_str!("Scene"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.tree_node(im_str!("Tree")).build(|| {
                    let live_entities = ecs.nb_entities();
                    for i in live_entities {

                        let selected = {
                            if let Some(selected) = self.selected_entity {
                                selected == i
                            } else { false }
                        };
                        if ui.selectable(
                            im_str!("Entity {}", i),
                            selected,
                            ImGuiSelectableFlags::empty(),
                            ImVec2::new(0.0, 0.0)) {

                            self.selected_entity = Some(i);
                        }
                    }

                });
            });

        true
    }
}
