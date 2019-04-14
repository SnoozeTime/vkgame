use super::Editor;
use crate::event::{EditorEvent, Event};
use imgui::{im_str, ImGuiCond, Ui};

/// Show a window where the user can turn on/off the game
/// configuration
pub fn show_configuration_window(ui: &Ui, editor: &mut Editor) {
    ui.window(im_str!("Configuration"))
        .size((300.0, 100.0), ImGuiCond::FirstUseEver)
        .build(|| {
            ui.tree_node(im_str!("Renderer"))
                .opened(true, ImGuiCond::FirstUseEver)
                .build(|| {
                    if ui.checkbox(
                        im_str!("Display outlines"),
                        &mut editor.game_config.renderer_config.display_outlines,
                    ) {
                        editor.event_to_process = Some(Event::EditorEvent(
                            EditorEvent::ConfigChange(editor.game_config),
                        ));
                    }
                    if ui.checkbox(
                        im_str!("Show shadow map"),
                        &mut editor.game_config.renderer_config.show_shadowmap,
                    ) {
                        editor.event_to_process = Some(Event::EditorEvent(
                            EditorEvent::ConfigChange(editor.game_config),
                        ));
                    }

                    if ui.checkbox(
                        im_str!("Show light-pov scene"),
                        &mut editor.game_config.renderer_config.show_shadowmap_color,
                    ) {
                        editor.event_to_process = Some(Event::EditorEvent(
                            EditorEvent::ConfigChange(editor.game_config),
                        ));
                    }
                });
            // Display each kind of configuration
            editor.hovered = ui.want_capture_mouse();

            if ui.button(im_str!("Close"), (0.0, 0.0)) {
                editor.hide_configuration_window();
            }
        });
}
