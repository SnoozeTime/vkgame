use vulkano::instance::Instance;
use winit::EventsLoop;
use twgraph::input::{KeyType, Input, Axis};

use std::time::Instant;

use twgraph::camera::{CameraDirection};
use twgraph::ecs::{
    ECS,
    systems::{DummySystem, RenderingSystem},
};
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

    // this is an Arc to instance. (non-mut dynamic ref)
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("Could not create instance")
    };

    // Get the surface and window. Window is from winit library
    let events_loop = EventsLoop::new();

    let mut ecs = get_ecs();
    let mut render_system = RenderingSystem::new(&instance, &events_loop);
    let mut dummy_system = DummySystem::new();
    let mut input = Input::new(events_loop);

    let mut old_instant = Instant::now();
    'game_loop: loop {

        // calculate frame time.
        let now = Instant::now();
        let frame_duration = now - old_instant;
        old_instant = now;

        render_system.render(&mut ecs, frame_duration);
        dummy_system.do_dumb_thing(frame_duration, &mut ecs);


        input.update(&mut render_system);

        // HANDLE CAMERA.
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


        // To quit
        if input.close_request || input.get_key_down(KeyType::Escape) {
            break 'game_loop;
        }
    }

}

