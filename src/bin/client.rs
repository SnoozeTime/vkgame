use clap::{App, Arg};
use log::info;
use std::time::{Duration, Instant};
use twgraph::input::{Input, KeyType};
use vulkano::instance::Instance;
use winit::EventsLoop;

use twgraph::ecs::systems::RenderingSystem;
use twgraph::resource::Resources;
use twgraph::scene::{ClientScene, SceneStack};

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

    info!("Start client: Will connect to {}", addr);
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

    let mut scenes = SceneStack::new();
    scenes.push(ClientScene::new(addr, &render_system));

    let fixed_time_stamp = Duration::new(0, 16666667);
    let mut previous_clock = Instant::now();
    let mut accumulator = Duration::new(0, 0);

    'game_loop: loop {
        while accumulator > fixed_time_stamp {
            accumulator -= fixed_time_stamp;
            // CHECK FOR RESOURCE UPDATE - I Guess this is just for dev purposes :D So
            // should find a flag to deactivate on release build.
            // See here https://doc.rust-lang.org/cargo/reference/manifest.html#the-profile-sections
            let events = resources.poll_events();
            render_system.handle_events(&events);

            let scene = scenes.get_current().unwrap();

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
            let _ = scene.process_input(Some(&input), Some(&resources), frame_duration);

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

        accumulator += Instant::now() - previous_clock;
        previous_clock = Instant::now();
    }

    info!("Bye bye");
}
