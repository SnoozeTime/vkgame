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
use twgraph::scene::{Scene, EditorScene, GameScene};
use twgraph::event::{Event, EditorEvent};
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


    let mut scenes: Vec<Box< dyn Scene>> = Vec::new();


    if let Some(_matches) = matches.subcommand_matches("game") {
        scenes.push(Box::new(GameScene::new(&render_system)));
    }

    if let Some(_matches) = matches.subcommand_matches("editor") {
        scenes.push(Box::new(EditorScene::new(&render_system)));
    }

    if scenes.len() == 0 {
        panic!("Need at least game or editor");
    }

    'game_loop: loop {

        let nb_scene = scenes.len();
        let scene = &mut scenes[nb_scene - 1];


        // calculate frame time.
        let now = Instant::now();
        let frame_duration = now - old_instant;
        old_instant = now;

        {
            let (ecs, gui) = scene.get_parts_mut();
            render_system.render(&resources,
                                 ecs,
                                 frame_duration,
                                 gui);
        }
        input.update(&mut render_system);

        // Now scene specific updates.
        scene.update(frame_duration);
        let events = scene.process_input(&input, &resources, frame_duration);

        if let Some(events) = events {
            if let Some(Event::EditorEvent(EditorEvent::PlayGame)) = events.get(0) {
                scenes.push(Box::new(GameScene::new(&render_system)));
            }
        }

        // To quit
        if input.close_request || input.get_key_down(KeyType::Escape) {
            break 'game_loop;
        }
    }

}

