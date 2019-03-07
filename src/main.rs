use vulkano::instance::Instance;
use clap::{App, SubCommand};
use winit::EventsLoop;
use twgraph::input::{KeyType, Input};

use std::time::Instant;

use twgraph::ecs::{
    ECS,
    systems::{RenderingSystem},
};
use twgraph::time::dt_as_secs;
use twgraph::resource::Resources;
use twgraph::scene::{Scene, EditorScene, GameScene};
use twgraph::event::{Event, EditorEvent};

fn main() {

    let matches = App::new("TwoWalker Game Engine")
        .version("0.1")
        .author("Benoit Eudier <benoit.eudier@gmail.com>")
        .subcommand(SubCommand::with_name("game"))
        .subcommand(SubCommand::with_name("editor"))
        .get_matches();
    // this is an Arc to instance. (non-mut dynamic ref)
    
        // NOTE: To simplify the example code we won't verify these layer(s) are actually in the
        // layers list:
         let layer = "VK_LAYER_LUNARG_standard_validation";
         let layers = vec![layer];
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, layers).expect("Could not create instance")
    };

    // Get the surface and window. Window is from winit library
    let events_loop = EventsLoop::new();

    let now = Instant::now();
    let mut render_system = RenderingSystem::new(&instance, &events_loop);
    let elapsed_render = Instant::now() - now;


    let now = Instant::now();
    let mut resources = Resources::new(
        render_system.get_device().clone(),
        render_system.get_queue().clone());
    let elapsed_resources = Instant::now() - now;

    println!("CREATE RENDERER: {}", dt_as_secs(elapsed_render));
    println!("CREATE RESOURCES: {}", dt_as_secs(elapsed_resources));
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
                // TODO copy the ECS
                let ecs = ECS::new_from_existing(scene.get_ecs());
                scenes.push(Box::new(GameScene::from_ecs(ecs, &render_system)));
            }
        }

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

