use clap::{App, Arg};
use log::{debug, info};
use std::thread;
use std::time::{Duration, Instant};
use twgraph::input::{Input, KeyType};
use vulkano::instance::Instance;
use winit::EventsLoop;

use twgraph::ecs::{systems::RenderingSystem, ECS};
use twgraph::event::{EditorEvent, Event};
use twgraph::resource::Resources;
use twgraph::scene::{ClientScene, EditorScene, Scene};

fn main() {
    env_logger::init();

    // Extract the server address from the command-line.
    let matches = App::new("Client")
        .version("0.1")
        .author("Benoit Eudier")
        .arg(
            Arg::with_name("connect")
                .short("c")
                .long("connect")
                .required(false)
                .takes_value(true)
                .help("IP address of the server"),
        )
        .get_matches();

    let addr = matches.value_of("connect").unwrap_or("localhost:8080");
    let layer = "VK_LAYER_LUNARG_standard_validation";
    let layers = vec![layer];
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

    let mut scenes: Vec<Box<dyn Scene>> = Vec::new();
    //    scenes.push(Box::new(EditorScene::new(&render_system, &resources)));
    scenes.push(Box::new(ClientScene::new(&render_system)));

    'game_loop: loop {
        // CHECK FOR RESOURCE UPDATE - I Guess this is just for dev purposes :D So
        // should find a flag to deactivate on release build.
        // See here https://doc.rust-lang.org/cargo/reference/manifest.html#the-profile-sections
        let events = resources.poll_events();
        render_system.handle_events(&events);

        let nb_scene = scenes.len();
        let scene = &mut scenes[nb_scene - 1];

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

        if input.get_key_down(KeyType::Escape) {
            let _ = scenes.pop();

            if scenes.len() == 0 {
                break 'game_loop;
            }
        }

        // To quit
        if input.close_request {
            break 'game_loop;
        }
    }
}
