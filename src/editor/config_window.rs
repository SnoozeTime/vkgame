use super::Editor;
use crate::event::{EditorEvent, Event};
use crate::renderer::DEBUG_ATTACHMENTS;
use imgui::{im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2, Ui};

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

                    // Check here to show one of the intermediate buffers to screen
                    if ui.checkbox(
                        im_str!("Show debug attachment"),
                        &mut editor.show_debug_attachment,
                    ) {
                        if !editor.show_debug_attachment {
                            editor.game_config.renderer_config.attachment_to_show = None;
                            editor.event_to_process = Some(Event::EditorEvent(
                                EditorEvent::ConfigChange(editor.game_config),
                            ));
                        }
                    }

                    if editor.show_debug_attachment {
                        for el in DEBUG_ATTACHMENTS.iter() {
                            let selected = editor
                                .game_config
                                .renderer_config
                                .attachment_to_show
                                .map_or(false, |ty| ty == *el);
                            if ui.selectable(
                                im_str!("{:?}", el),
                                selected,
                                ImGuiSelectableFlags::empty(),
                                ImVec2::new(0.0, 0.0),
                            ) {
                                editor.game_config.renderer_config.attachment_to_show = Some(*el);
                                editor.event_to_process = Some(Event::EditorEvent(
                                    EditorEvent::ConfigChange(editor.game_config),
                                ));
                            }
                        }
                    }
                });
            // Display each kind of configuration
            editor.hovered = ui.want_capture_mouse();

            if ui.button(im_str!("Close"), (0.0, 0.0)) {
                editor.hide_configuration_window();
            }
        });
}
