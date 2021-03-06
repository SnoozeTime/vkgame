use super::Editor;
use crate::ecs::ECS;
use imgui::{im_str, ImString, Ui};
use log::*;

#[derive(Debug, Copy, Clone)]
pub enum FileSelectAction {
    Load,
    Save,
}

pub struct FileSelect {
    pub action: Option<FileSelectAction>,
    pub label: String,
    pub buf: ImString,
}

impl FileSelect {
    pub fn new() -> Self {
        let buf = ImString::with_capacity(32);
        FileSelect {
            action: None,
            label: String::new(),
            buf,
        }
    }

    pub fn set_save_file(&mut self) {
        self.label = String::from("Save scene");
        self.action = Some(FileSelectAction::Save);
    }

    pub fn set_load_file(&mut self) {
        self.label = String::from("Load scene");
        self.action = Some(FileSelectAction::Load);
    }
}

fn execute(editor: &mut Editor, ecs: &mut ECS) {
    let action = editor.file_select.action.unwrap(); // that would be a big coding mistake if crash here :D
    let filename = { String::from(editor.file_select.buf.to_str()) };
    let res = match action {
        FileSelectAction::Save => editor.save(ecs, filename),
        FileSelectAction::Load => editor.load(ecs, filename),
    };
}

pub fn file_select(ui: &Ui, ecs: &mut ECS, editor: &mut Editor) {
    ui.window(im_str!("FileSelect")).build(|| {
        editor.hovered = ui.want_capture_mouse();

        if ui
            .input_text(im_str!(""), &mut editor.file_select.buf)
            .enter_returns_true(true) // only pressing enter will trigger the action
            .build()
        {
            execute(editor, ecs);

            editor.file_select.action = None;
            editor.show_fileselect = false;
        }

        if ui.button(im_str!("{}", editor.file_select.label), (0.0, 0.0)) {
            execute(editor, ecs);
            editor.file_select.action = None;
            editor.show_fileselect = false;
        }

        if ui.button(im_str!("Close"), (0.0, 0.0)) {
            editor.file_select.action = None;
            editor.show_fileselect = false;
        }
    });
}
