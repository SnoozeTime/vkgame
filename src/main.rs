use vulkano::sync::GpuFuture;
use vulkano::instance::Instance;
use vulkano::format::Format;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::image::{ImmutableImage, Dimensions, SwapchainImage};
use vulkano_win::VkSurfaceBuild;
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano_win;
use winit::{KeyboardInput, VirtualKeyCode, EventsLoop, WindowBuilder, Event, WindowEvent};
use cgmath::{Point3, Vector3};

use image::ImageFormat;
use std::sync::Arc;
use std::time::Instant;

use twgraph::camera::{CameraDirection, Camera};
use twgraph::gameobject::{Scene, Transform, MeshComponent};
use twgraph::render::RenderSystem;
use twgraph::model::Model;

fn new_scene() -> Scene {
    let camera_transform = Transform {
        position: Point3::new(0.0, 0.0, 1.0),
        rotation: Vector3::new(0.0, 0.0, 0.0),
        scale: Point3::new(0.0, 0.0, 0.0),
    };
    let camera = Camera::new(camera_transform);

    let mesh_components = MeshComponent {
        mesh_name: Some("cube".to_owned()),
        texture_name: Some("The texture".to_owned()),
    };

    let transforms = Transform {
        position: Point3::new(0.0, 0.0, 1.0),
        rotation: Vector3::new(0.0, 0.0, 0.0),
        scale: Point3::new(0.0, 0.0, 0.0),
    };

    Scene {
        transforms,
        mesh_components,
        camera,
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

    let box_obj = Model::load_from_obj(render_system.device.clone(), "cube.obj").unwrap();
    let (texture, tex_future) = {
        let image = image::load_from_memory_with_format(include_bytes!("image_img.png"),
            ImageFormat::PNG).unwrap().to_rgba();
        let image_data = image.into_raw().clone();

        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            Dimensions::Dim2d { width: 93, height: 93 },
            Format::R8G8B8A8Srgb,
            render_system.queue.clone()
        ).unwrap()
    };


    let sampler = Sampler::new(
        render_system.device.clone(),
        Filter::Linear,
        Filter::Linear,
        MipmapMode::Nearest,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat,
        SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();
    let tex_set = Arc::new(PersistentDescriptorSet::start(render_system.pipeline.pipeline.clone(), 1)
        .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
        .build().unwrap()
    );


    let rotation_start = Instant::now();
    let mut scene = new_scene();

    render_system.previous_frame_end.take().unwrap();
    render_system.previous_frame_end = Some(Box::new(tex_future) as Box<GpuFuture>);
    loop {

         
        render_system.render(rotation_start.elapsed(), &scene, &box_obj, tex_set.clone());
    
        let mut done = false;
        events_loop.poll_events(|ev| {
            if let Event::WindowEvent { event, ..} = ev {
                match event {
                    WindowEvent::CloseRequested => done = true,
                    WindowEvent::Resized(_) => render_system.recreate_swapchain = true,
                    WindowEvent::CursorMoved { position: position, ..} => {
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
                            _ => (),
                        }
                    },
                            _ => (),
                }
            }});

    
           if done { return; }

    }

}

