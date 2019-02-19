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
use twgraph::scene::{Scene, EditorScene};
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

    let mut render_system = RenderingSystem::new(&instance, &events_loop);
    let mut resources = Resources::new(
        render_system.get_device().clone(),
        render_system.get_queue().clone());
    let mut input = Input::new(events_loop);

    let mut old_instant = Instant::now();

    let mut scene = EditorScene::new(&render_system);
    // Apply aspect to camera.
    {
        let dimensions = render_system.dimensions();
        scene.ecs.camera.set_aspect((dimensions[0] as f32) / (dimensions[1] as f32));
    }
    //
    //

    'game_loop: loop {

        // calculate frame time.
        let now = Instant::now();
        let frame_duration = now - old_instant;
        old_instant = now;

        render_system.render(&resources,
                             &mut scene.ecs,
                             frame_duration,
                             &mut scene.editor);

        input.update(&mut render_system);

        // Now scene specific updates.
        scene.update(frame_duration);
        scene.process_input(&input, &resources, frame_duration);

        // To quit
        if input.close_request || input.get_key_down(KeyType::Escape) {
            break 'game_loop;
        }
    }

}

