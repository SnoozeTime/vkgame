use imgui::{Ui, im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2, ImString};
use crate::ecs::{
    ECS,
    Entity,
    new_component_popup,
    gen_index::GenerationalIndexArray,
    components::{NameComponent, LightType},
};
use crate::ui::Gui;
use std::collections::HashMap;
mod file_select;
use file_select::{file_select, FileSelect};
use crate::resource::Resources;


type EntityNames = GenerationalIndexArray<String>;

pub struct Editor {
    pub selected_entity: Option<Entity>,
    pub hovered: bool,

    // For the component creation popup
    pub should_add_comp: bool,
    pub new_component_name: Option<String>,

    // For the components
    // dirty :D
    pub components_state: HashMap<String, String>,

    show_fileselect: bool,
    file_select: FileSelect,

    pub all_textures: Vec<String>,
    pub all_models: Vec<String>,

    // For the renaming
    pub rename_entity_buf: ImString, 



}

impl Editor {

    pub fn new(resources: &Resources) -> Self {

        let mut all_models = vec![];
        for (model_name, _) in &resources.models.models {
            all_models.push((*model_name).clone());
        }


        let mut all_textures = vec![];
        for (texture_name, _) in &resources.textures.textures {
            all_textures.push((*texture_name).clone());
        }

        let rename_entity_buf = ImString::with_capacity(32);

        Editor {
            selected_entity: None,
            hovered: false,
            new_component_name: None,
            should_add_comp: false,
            components_state: HashMap::new(),
            show_fileselect: false,
            file_select: FileSelect::new(),
            all_textures,
            all_models,
            rename_entity_buf,
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

fn display_menu(ui: &Ui, editor: &mut Editor, ecs: &mut ECS) {
    let mut open_delete_popup = false;
    let mut open_rename_popup = false;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("File")).build(|| {
            editor.hovered = true;
            if ui.menu_item(im_str!("Save"))
                .shortcut(im_str!("CTRL+S"))
                    .build() {
                        editor.file_select.set_save_file(); 
                        editor.show_fileselect = true;
                    }
            if ui.menu_item(im_str!("Load"))
                .shortcut(im_str!("CTRL+L"))
                    .build() {
                        editor.file_select.set_load_file(); 
                        editor.show_fileselect = true;
                    }
        });

        ui.menu(im_str!("Edit")).build(|| {
            editor.hovered = true;
            if ui.menu_item(im_str!("New entity"))
                .build() {
                    let entity = ecs.new_entity();
                    editor.select_entity(entity, ecs);
                }
            if ui.menu_item(im_str!("Delete entity"))
                .enabled(editor.selected_entity.is_some())
                    .build() {

                        open_delete_popup = true;
                        //new_component_popup(ui, self);
                    }

            if ui.menu_item(im_str!("Rename entity"))
                .enabled(editor.selected_entity.is_some())
                    .build() {
                        open_rename_popup = true;
                    }
        });

    });

    if open_delete_popup {
        ui.open_popup(im_str!("delete_entity_popup"));
    }
    ui.popup_modal(im_str!("delete_entity_popup")).always_auto_resize(true).build(|| {

        ui.text(im_str!("Are you sure you want to delete that entity?"));
        if ui.button(im_str!("Yes"), (0.0, 0.0)) {
            
            if let Some(entity) = &editor.selected_entity {
                ecs.delete_entity(entity);
                ui.close_current_popup();
            }
        }

        if ui.button(im_str!("No"), (0.0, 0.0)) {
            ui.close_current_popup();
        }
    });
    if open_rename_popup {
        ui.open_popup(im_str!("rename_entity_popup"));
    }
    ui.popup_modal(im_str!("rename_entity_popup")).always_auto_resize(true).build(|| {

        if ui.input_text(im_str!(""),
        &mut editor.rename_entity_buf)
            .enter_returns_true(true)
                .build() {

                    let new_name = String::from(editor.rename_entity_buf.to_str());
                    // cannot fail here as is_some returned true.
                    ecs.components.names.set(
                        &editor.selected_entity.unwrap(),
                        NameComponent { name: new_name});
                    editor.rename_entity_buf.clear();
                    ui.close_current_popup();
                }

        if ui.button(im_str!("Rename"), (0.0, 0.0)) {

            let new_name = String::from(editor.rename_entity_buf.to_str());
            // cannot fail here as is_some returned true.
            ecs.components.names.set(
                &editor.selected_entity.unwrap(),
                NameComponent { name: new_name });
            editor.rename_entity_buf.clear();
            ui.close_current_popup();
        }

        if ui.button(im_str!("Close"), (0.0, 0.0)) {

            editor.rename_entity_buf.clear();
            ui.close_current_popup();
        }

    });


    if editor.show_fileselect {
        file_select(ui, ecs, editor);
    }


}

impl Gui for Editor {
    /// This is the function that will create the GUI!
    fn run_ui(&mut self,
              ui: &Ui,
              ecs: &mut ECS) -> bool {

        // Should be first. Reset the state
        self.hovered = false;

        display_menu(ui, self, ecs);
        // That is our tree !
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
                    let mut entity_name = &format!("Entity {}", i);
                    if let Some(NameComponent { ref name }) = ecs.components.names.get(entity) {
                        entity_name = name;
                    }

                    if ui.selectable(
                        im_str!("{}", entity_name),
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
