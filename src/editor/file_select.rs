use imgui::{Ui, im_str, ImGuiCond, ImGuiSelectableFlags, ImVec2, ImString};
use crate::ecs::{
    ECS,
};
use super::Editor;

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

fn execute(action: FileSelectAction, filename: &str, ecs: &mut ECS) {

    let res = match action {
        FileSelectAction::Save => {
            ecs.save(filename)
        },
        FileSelectAction::Load => {
            ecs.load_and_replace(filename)
        }
    };

    if let Err(err) = res {
        println!("{:?}", err);
    }

}

pub fn file_select(ui: &Ui, ecs: &mut ECS, editor: &mut Editor) {
    ui.window(im_str!("FileSelect"))
        .build(|| {

            editor.hovered = ui.want_capture_mouse();

            if ui.input_text(im_str!(""), &mut editor.file_select.buf)
                .enter_returns_true(true) // only pressing enter will trigger the action
                .build() {
                execute(editor.file_select.action.unwrap(), // that would be a big coding mistake if crash here :D
                        editor.file_select.buf.to_str(), 
                        ecs);

                editor.file_select.action = None; 
                editor.show_fileselect = false;
            }

            if ui.button(im_str!("{}", editor.file_select.label), (0.0, 0.0)) {
                execute(editor.file_select.action.unwrap(), // that would be a big coding mistake if crash here :D
                        editor.file_select.buf.to_str(), 
                        ecs);
                editor.file_select.action = None; 
                editor.show_fileselect = false;
            }


            if ui.button(im_str!("Close"),
            (0.0, 0.0)) {
                editor.file_select.action = None;
                editor.show_fileselect = false;
            }

        });

}

