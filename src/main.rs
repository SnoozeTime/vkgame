use vulkano::instance::Instance;
use clap::{Arg, App, SubCommand};
use winit::EventsLoop;
use twgraph::input::{KeyType, Input, Axis, MouseButton};

use std::time::Instant;

use twgraph::camera::{CameraDirection};
use twgraph::ecs::{
    ECS,
    systems::{DummySystem, RenderingSystem},
};
use twgraph::editor::Editor;
use twgraph::resource::Resources;
use std::env;

fn get_ecs() -> ECS {
    let mut args = env::args();
    if let Some(path) = args.nth(1) {
        println!("will load: {:?}", path);
        ECS::load(path).unwrap()
    } else {
        ECS::dummy_ecs()
    }
}

fn main() {

    let matches = App::new("TwoWalker Game Engine")
        .version("0.1")
        .author("Benoit Eudier <benoit.eudier@gmail.com>")
        .subcommand(SubCommand::with_name("game"))
        .subcommand(SubCommand::with_name("editor"))
        .get_matches();


    if let Some(matches) = matches.subcommand_matches("game") {

    }

    if let Some(matches) = matches.subcommand_matches("editor") {

    }

    // this is an Arc to instance. (non-mut dynamic ref)
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("Could not create instance")
    };

    // Get the surface and window. Window is from winit library
    let events_loop = EventsLoop::new();

    let mut ecs = get_ecs();
    let mut render_system = RenderingSystem::new(&instance, &events_loop);
    let mut resources = Resources::new(
        render_system.get_device().clone(),
        render_system.get_queue().clone());
    let mut dummy_system = DummySystem::new();
    let mut input = Input::new(events_loop);

    let mut old_instant = Instant::now();

    let mut editor = Editor::new();

    // Apply aspect to camera.
    {
        let dimensions = render_system.dimensions();
        ecs.camera.set_aspect((dimensions[0] as f32) / (dimensions[1] as f32));
    }
    //
    'game_loop: loop {

        // calculate frame time.
        let now = Instant::now();
        let frame_duration = now - old_instant;
        old_instant = now;

        render_system.render(&resources, &mut ecs, frame_duration, |ui, ecs| {
            editor.run_ui(ui, ecs)
        });

        dummy_system.do_dumb_thing(frame_duration, &mut ecs);

        input.update(&mut render_system);

        // HANDLE CAMERA.
        if input.modifiers.ctrl {
            if input.get_key(KeyType::Up) {
                ecs.camera.process_keyboard(frame_duration,
                                            CameraDirection::Forward);
            }

            if input.get_key(KeyType::Down) {
                ecs.camera.process_keyboard(frame_duration,
                                            CameraDirection::Backward);
            }

            if input.get_key(KeyType::Left) {
                ecs.camera.process_keyboard(frame_duration,
                                            CameraDirection::Left);
            }

            if input.get_key(KeyType::Right) {
                ecs.camera.process_keyboard(frame_duration,
                                            CameraDirection::Right);
            }

            let (h_axis, v_axis) = (input.get_axis(Axis::Horizontal),
            input.get_axis(Axis::Vertical));
            if h_axis != 0.0 || v_axis != 0.0 {
                ecs.camera.process_mouse(frame_duration,
                                         h_axis,
                                         v_axis);
            }
        }

        if input.get_mouse_clicked(MouseButton::Left) && !editor.hovered {
            editor.selected_entity = render_system.pick_object(input.mouse_pos[0],
                                                          input.mouse_pos[1],
                                                          &ecs,
                                                          &resources);
        }

        // To quit
        if input.close_request || input.get_key_down(KeyType::Escape) {
            break 'game_loop;
        }
    }

}

