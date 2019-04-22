use clap::{App, Arg};
use std::time::Instant;
use twgraph::input::{Input, KeyType};
use vulkano::instance::Instance;
use winit::EventsLoop;

use twgraph::ecs::{systems::RenderingSystem, ECS};
use twgraph::event::{EditorEvent, Event, GameEvent};
use twgraph::resource::Resources;
use twgraph::scene::{EditorScene, GameScene, SceneStack};

fn main() {
    env_logger::init();

    let matches = App::new("Editor")
        .version("0.1")
        .arg(
            Arg::with_name("scene")
                .short("s")
                .long("scene")
                .required(false)
                .takes_value(true)
                .help("JSON file that represent a scene"),
        )
        .get_matches();

    //    let layer = "VK_LAYER_LUNARG_standard_validation";
    //
    //    let layers = vec![layer];
    let layers = vec![];
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, layers).expect("Could not create instance")
    };

    // Get the surface and window. Window is from winit library
    let events_loop = EventsLoop::new();

    let mut render_system = RenderingSystem::new(&instance, &events_loop);
    let mut resources = Resources::new(render_system.get_queue().clone());

    let mut input = Input::new(events_loop);
    let mut old_instant = Instant::now();

    let mut scenes = SceneStack::new();

    // Editor scene depends on the input arguments
    let editor_scene = if let Some(scene) = matches.value_of("scene") {
        EditorScene::from_path(scene.to_string(), &render_system, &resources)
    } else {
        EditorScene::new(&render_system, &resources)
    };
    scenes.push(editor_scene);

    'game_loop: loop {
        // CHECK FOR RESOURCE UPDATE - I Guess this is just for dev purposes :D So
        // should find a flag to deactivate on release build.
        // See here https://doc.rust-lang.org/cargo/reference/manifest.html#the-profile-sections
        if scenes.len() == 0 {
            break 'game_loop;
        }
        let scene = scenes
            .get_current()
            .expect("A scene should be in the stack");

        let events = resources.poll_events();
        render_system.handle_events(&events);

        // calculate frame time.
        let now = Instant::now();
        let frame_duration = now - old_instant;
        old_instant = now;

        {
            let (ecs, gui) = scene.get_parts_mut();
            render_system.render(&resources, ecs, frame_duration, gui.unwrap());
        }
        input.update(&mut render_system);

        // Now scene specific updates.
        scene.update(frame_duration);
        let events = scene.process_input(Some(&input), Some(&resources), frame_duration);

        let mut should_quit = false;
        let mut start_game = false;
        if let Some(events) = events {
            for event in events.iter() {
                match *event {
                    Event::EditorEvent(EditorEvent::PlayGame) => {
                        start_game = true;
                    }
                    Event::EditorEvent(EditorEvent::QuitEditor)
                    | Event::GameEvent(GameEvent::QuitGame) => {
                        should_quit = true;
                    }
                    _ => (),
                }
            }

            render_system.handle_events(&events);
        }

        if start_game {
            let new_scene = {
                let ecs = scene.get_ecs();
                GameScene::from_ecs(ECS::new_from_existing(ecs), &render_system)
            };
            scenes.push(new_scene);
        }

        // To quit
        if input.close_request || should_quit {
            let _ = scenes.pop();
            if scenes.len() == 0 {
                break 'game_loop;
            }
        }
    }
}
