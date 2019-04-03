use std::time::Instant;
use twgraph::input::{Input, KeyType};
use vulkano::instance::Instance;
use winit::EventsLoop;

use twgraph::ecs::{systems::RenderingSystem, ECS};
use twgraph::event::{EditorEvent, Event};
use twgraph::resource::Resources;
use twgraph::scene::{EditorScene, GameScene, SceneStack};

fn main() {
    env_logger::init();
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
    scenes.push(EditorScene::new(&render_system, &resources));

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

        if let Some(events) = events {
            if let Some(Event::EditorEvent(EditorEvent::PlayGame)) = events.get(0) {
                // TODO copy the ECS
                let ecs = ECS::new_from_existing(scene.get_ecs());
                scenes.push(GameScene::from_ecs(ecs, &render_system));
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
