use vulkano::instance::Instance;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;
use winit::{KeyboardInput, VirtualKeyCode, EventsLoop, WindowBuilder, Event, WindowEvent};

use twgraph::camera::{CameraDirection};
use twgraph::gameobject::{Scene};
use twgraph::render::RenderSystem;
use std::env;

fn get_scene() -> Scene {
    let mut args = env::args();
    if let Some(path) = args.nth(1) {
        println!("will load: {:?}", path);
        Scene::load(path).unwrap()
    } else {
        Scene::new_dummy()
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
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .expect("Cannot create vk_surface");
    let _window = surface.window();
    let mut render_system = RenderSystem::new(&instance, surface.clone()).unwrap();
    render_system.load_texture("bonjour".to_string(),
        std::path::Path::new("src/image_img.png"),
        93, 93).unwrap();
    render_system.load_model("cube".to_string(), std::path::Path::new("cube.obj")).expect("Cannot load model");

    //let rotation_start = Instant::now();
    let mut scene = get_scene(); 

    loop {
        render_system.render(&scene);
    
        let mut done = false;
        events_loop.poll_events(|ev| {
            if let Event::WindowEvent { event, ..} = ev {
                match event {
                    WindowEvent::CloseRequested => done = true,
                    WindowEvent::Resized(_) => render_system.recreate_swapchain = true,
                    WindowEvent::CursorMoved { position, ..} => {
                        scene.camera.process_mouse(position.x, position.y);
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
                            VirtualKeyCode::W => scene.camera.process_keyboard(CameraDirection::Forward),
                            VirtualKeyCode::S => scene.camera.process_keyboard(CameraDirection::Backward),
                            VirtualKeyCode::A => scene.camera.process_keyboard(CameraDirection::Left),
                            VirtualKeyCode::D => scene.camera.process_keyboard(CameraDirection::Right),
                            VirtualKeyCode::Space => {
                                match scene.save("scene.json".to_owned()) {
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

