use vulkano::instance::Instance;
use winit::{KeyboardInput, VirtualKeyCode, EventsLoop, Event, WindowEvent};
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
    let mut events_loop = EventsLoop::new();

    let mut ecs = get_ecs();
    let mut render_system = RenderingSystem::new(&instance, &events_loop);
    let mut dummy_system = DummySystem{};
    
    let mut old_instant = Instant::now();
    loop {

        // calculate frame time.
        let now = Instant::now();
        let frame_duration = now - old_instant;
        old_instant = now;

        render_system.render(&ecs);
        dummy_system.do_dumb_thing(frame_duration, &mut ecs);

        let mut done = false;
        events_loop.poll_events(|ev| {
            if let Event::WindowEvent { event, ..} = ev {
                match event {
                    WindowEvent::CloseRequested => done = true,
                    WindowEvent::Resized(_) => render_system.resize_window(),
                    WindowEvent::CursorMoved { position, ..} => {
                        ecs.camera.process_mouse(frame_duration, position.x, position.y);
                    },
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                            ..
                    } => {
                        match keycode {
                            VirtualKeyCode::Escape => done = true,
                            VirtualKeyCode::W => ecs.camera.process_keyboard(frame_duration,
                                                                             CameraDirection::Forward),
                            VirtualKeyCode::S => ecs.camera.process_keyboard(frame_duration,
                                                                             CameraDirection::Backward),
                            VirtualKeyCode::A => ecs.camera.process_keyboard(frame_duration,
                                                                             CameraDirection::Left),
                            VirtualKeyCode::D => ecs.camera.process_keyboard(frame_duration,
                                                                             CameraDirection::Right),
                            VirtualKeyCode::Space => {
                                match ecs.save("scene.json".to_owned()) {
                                    Ok(_) => println!("Successfully saved scene.json"),
                                    Err(err) => println!("{}", err),
                                }
                            },
                            _ => (),
                        }
                    },
                            _ => (),
                }
            }});


        if done { return; }
    }

}

