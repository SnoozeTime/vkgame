use crate::ecs::{
    components::{LightType, NameComponent},
    Entity, ECS,
};
use crate::event::{EditorEvent, Event};
use crate::ui::Gui;
use imgui::{im_str, ImGuiCond, ImGuiSelectableFlags, ImString, ImVec2, Ui};
use log::*;
use std::collections::HashMap;
use std::path::PathBuf;
mod file_select;
use crate::resource::Resources;
use file_select::{file_select, FileSelect};

pub struct Editor {
    pub scene_names: Vec<PathBuf>,
    pub current_scene_idx: Option<usize>,

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

    // Prompt before quitting without saving.
    pub unsaved: bool,
    show_confirmation_prompt: bool,

    // User request something to the editor but editor need to wait for some confirmation
    // e.g. quit the editor, but some unsaved state.
    pub pending_event: Option<Event>,
    // once pending event is confirmed or canceld, would be store here
    pub event_to_process: Option<Event>,
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
            scene_names: resources.get_scene_names(),
            current_scene_idx: None,
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
            unsaved: false,
            show_confirmation_prompt: false,
            pending_event: None,
            event_to_process: None,
        }
    }

    pub fn save(&mut self, ecs: &mut ECS, filename: String) {
        if let Err(e) = ecs.save(&filename) {
            error!("Error while saving {} = {:?}", filename, e);
        } else {
            self.set_saved();
            self.update_scene_names(filename);
        }
    }

    pub fn load(&mut self, ecs: &mut ECS, filename: String) {
        if let Err(e) = ecs.load_and_replace(&filename) {
            error!("Error while saving {} = {:?}", filename, e);
        } else {
            self.set_saved();
            self.update_scene_names(filename);
        }
    }

    pub fn update_scene_names(&mut self, filename: String) {
        let filename = PathBuf::from(filename);
        if let Some(i) = self
            .scene_names
            .iter()
            .enumerate()
            .find(|(_, p)| **p == filename)
            .map(|t| t.0)
        {
            self.current_scene_idx = Some(i);
        } else {
            self.scene_names.push(filename);
            self.current_scene_idx = Some(self.scene_names.len() - 1);
        }
    }

    pub fn next_scene(&mut self, ecs: &mut ECS) {
        if let Some(idx) = self.current_scene_idx {
            if self.scene_names.len() > 1 {
                self.load_scene_by_idx(ecs, (idx + 1) % (self.scene_names.len()));
            }
        }
    }

    pub fn previous_scene(&mut self, ecs: &mut ECS) {
        if let Some(idx) = self.current_scene_idx {
            if self.scene_names.len() > 1 {
                let idx = if idx == 0 {
                    self.scene_names.len() - 1
                } else {
                    idx - 1
                };

                self.load_scene_by_idx(ecs, idx);
            }
        }
    }

    fn load_scene_by_idx(&mut self, ecs: &mut ECS, idx: usize) {
        if let Err(e) = ecs.load_and_replace(&self.scene_names[idx]) {
            error!("Error while saving {:?} = {:?}", self.scene_names[idx], e);
        } else {
            self.set_saved();
            self.current_scene_idx = Some(idx);
        }
    }

    pub fn show_confirmation_prompt(&mut self) {
        self.show_confirmation_prompt = true;
    }

    pub fn filename(&self) -> &str {
        if let Some(idx) = self.current_scene_idx {
            self.scene_names[idx].to_str().unwrap()
        } else {
            "Untitled"
        }
    }

    fn select_entity(&mut self, entity: Entity, ecs: &ECS) {
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

    pub fn set_unsaved(&mut self) {
        self.unsaved = true;
    }

    pub fn set_saved(&mut self) {
        self.unsaved = false;
    }

    pub fn request_quit(&mut self) {
        if self.unsaved {
            self.pending_event = Some(Event::EditorEvent(EditorEvent::QuitEditor));
            self.show_confirmation_prompt();
        } else {
            self.event_to_process = Some(Event::EditorEvent(EditorEvent::QuitEditor));
        }
    }

    pub fn request_load_next(&mut self) {
        if self.unsaved {
            self.pending_event = Some(Event::EditorEvent(EditorEvent::LoadNext));
            self.show_confirmation_prompt();
        } else {
            self.event_to_process = Some(Event::EditorEvent(EditorEvent::LoadNext));
        }
    }

    pub fn request_load_previous(&mut self) {
        if self.unsaved {
            self.pending_event = Some(Event::EditorEvent(EditorEvent::LoadPrevious));
            self.show_confirmation_prompt();
        } else {
            self.event_to_process = Some(Event::EditorEvent(EditorEvent::LoadPrevious));
        }
    }
}

/*
 * This will be shown if the level is not saved yet but the user quits
 * or tries to load another level
 * */
fn display_confirmation_popup(ui: &Ui, editor: &mut Editor, ecs: &mut ECS) {
    if editor.show_confirmation_prompt {
        ui.open_popup(im_str!("confirmation_prompt"));
    }
    ui.popup_modal(im_str!("confirmation_prompt"))
        .always_auto_resize(true)
        .build(|| {
            ui.text(im_str!(
                "Will apply: {:?}. Do you want to save before? (File name: {})",
                editor.pending_event,
                editor.filename()
            ));
            if ui.button(im_str!("Yes"), (0.0, 0.0)) {
                editor.set_saved();
                ecs.save(editor.filename());
                editor.event_to_process = editor.pending_event.take();
                editor.show_confirmation_prompt = false;
                ui.close_current_popup();
            }
            if ui.button(im_str!("No"), (0.0, 0.0)) {
                editor.event_to_process = editor.pending_event.take();
                editor.show_confirmation_prompt = false;
                ui.close_current_popup();
            }
            if ui.button(im_str!("Cancel"), (0.0, 0.0)) {
                editor.pending_event.take();
                editor.show_confirmation_prompt = false;
                ui.close_current_popup();
            }
        });
}

fn display_menu(ui: &Ui, editor: &mut Editor, ecs: &mut ECS) {
    let mut open_delete_popup = false;
    let mut open_rename_popup = false;

    ui.main_menu_bar(|| {
        ui.menu(im_str!("File")).build(|| {
            editor.hovered = true;
            if ui
                .menu_item(im_str!("Save"))
                .shortcut(im_str!("CTRL+S"))
                .build()
            {
                editor.file_select.set_save_file();
                editor.show_fileselect = true;
            }
            if ui
                .menu_item(im_str!("Load"))
                .shortcut(im_str!("CTRL+L"))
                .build()
            {
                editor.file_select.set_load_file();
                editor.show_fileselect = true;
            }
        });

        ui.menu(im_str!("Edit")).build(|| {
            editor.hovered = true;
            if ui.menu_item(im_str!("New entity")).build() {
                let entity = ecs.new_entity();
                editor.select_entity(entity, ecs);
                editor.set_unsaved();
            }

            if ui.menu_item(im_str!("New entity at position")).build() {
                let entity = ecs.new_entity();
                editor.select_entity(entity, ecs);
                let t = (*ecs.camera.transform()).clone();
                ecs.components.transforms.set(&entity, t);
                editor.set_unsaved();
            }
            if ui
                .menu_item(im_str!("Delete entity"))
                .enabled(editor.selected_entity.is_some())
                .build()
            {
                open_delete_popup = true;
            }

            if ui
                .menu_item(im_str!("Rename entity"))
                .enabled(editor.selected_entity.is_some())
                .build()
            {
                open_rename_popup = true;
            }
        });

        ui.text(im_str!("Current file name: {}", editor.filename()));
    });

    if open_delete_popup {
        ui.open_popup(im_str!("delete_entity_popup"));
    }
    ui.popup_modal(im_str!("delete_entity_popup"))
        .always_auto_resize(true)
        .build(|| {
            ui.text(im_str!("Are you sure you want to delete that entity?"));
            if ui.button(im_str!("Yes"), (0.0, 0.0)) {
                if let Some(entity) = &editor.selected_entity {
                    ecs.delete_entity(entity);
                    editor.set_unsaved();
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
    ui.popup_modal(im_str!("rename_entity_popup"))
        .always_auto_resize(true)
        .build(|| {
            if ui
                .input_text(im_str!(""), &mut editor.rename_entity_buf)
                .enter_returns_true(true)
                .build()
            {
                let new_name = String::from(editor.rename_entity_buf.to_str());
                // cannot fail here as is_some returned true.
                ecs.components.names.set(
                    &editor.selected_entity.unwrap(),
                    NameComponent { name: new_name },
                );
                editor.rename_entity_buf.clear();
                editor.set_unsaved();
                ui.close_current_popup();
            }

            if ui.button(im_str!("Rename"), (0.0, 0.0)) {
                let new_name = String::from(editor.rename_entity_buf.to_str());
                // cannot fail here as is_some returned true.
                ecs.components.names.set(
                    &editor.selected_entity.unwrap(),
                    NameComponent { name: new_name },
                );
                editor.rename_entity_buf.clear();
                editor.set_unsaved();
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
    fn run_ui(&mut self, ui: &Ui, ecs: &mut ECS) -> bool {
        // Should be first. Reset the state
        self.hovered = false;

        display_menu(ui, self, ecs);
        display_confirmation_popup(ui, self, ecs);

        // That is our tree !
        ui.window(im_str!("Scene"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                let live_entities = ecs.nb_entities();
                for (i, entity) in live_entities.iter().enumerate() {
                    let selected = {
                        if let Some(selected) = self.selected_entity {
                            selected.index() == i
                        } else {
                            false
                        }
                    };
                    let mut entity_name = &format!("Entity {}", i);
                    if let Some(NameComponent { ref name }) = ecs.components.names.get(entity) {
                        entity_name = name;
                    }

                    if ui.selectable(
                        im_str!("{}", entity_name),
                        selected,
                        ImGuiSelectableFlags::empty(),
                        ImVec2::new(0.0, 0.0),
                    ) {
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
                    ecs.add_new_component_by_name(&self.selected_entity, &self.new_component_name);
                    self.should_add_comp = false;
                    self.new_component_name = None;
                    self.set_unsaved();
                }
            });

        self.hovered = ui.want_capture_mouse();

        true
    }
}
