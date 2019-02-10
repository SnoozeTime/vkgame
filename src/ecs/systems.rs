use winit::{WindowBuilder};
use vulkano::instance::Instance;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;

use std::sync::Arc;
use std::path::Path;
use std::time::Duration;

use crate::renderer::Renderer;
use crate::time::dt_as_secs;

use super::ECS;

pub struct RenderingSystem<'a> {
    renderer: Renderer<'a>,    
}

impl<'a> RenderingSystem<'a> {

    pub fn new(instance: &'a Arc<Instance>, events_loop: &winit::EventsLoop) -> Self {
        // Get the surface and window. Window is from winit library
        let surface = WindowBuilder::new()
            .with_dimensions((600, 600).into())
            .with_resizable(false)
            .build_vk_surface(&events_loop, instance.clone())
            .expect("Cannot create vk_surface");

        let window = surface.window();
        window.grab_cursor(true).unwrap();

        // TODO error handling
        let mut renderer = Renderer::new(&instance, surface.clone()).unwrap();

        Self::init_textures(&mut renderer);
        Self::init_models(&mut renderer);

        RenderingSystem {
            renderer,
        }
    }

    pub fn resize_window(&mut self) {
        self.renderer.recreate_swapchain = true;
    }

    fn init_textures(render_system: &mut Renderer) {
        render_system.load_texture("bonjour".to_string(),
        Path::new("assets/image_img.png"),
        93, 93).unwrap();
        render_system.load_texture("white".to_string(), Path::new("assets/white.png"), 93, 93).unwrap();
        render_system.load_texture("red".to_string(), Path::new("assets/red.png"), 93, 93).unwrap();
        render_system.load_texture("blue".to_string(), Path::new("assets/blue.png"), 93, 93).unwrap();
        render_system.load_texture("green".to_string(), Path::new("assets/green.png"), 93, 93).unwrap();
        render_system.load_texture("floor".to_string(), Path::new("assets/textures/Concrete_Panels_001_COLOR.jpg"), 1024, 1024).unwrap();
    //render_system.load_texture("chalet".to_string(),
        //Path::new("chalet.jpg"),
       // 4096, 4096).unwrap();
    }

    fn init_models(render_system: &mut Renderer) {
        println!("Init models!");
        render_system.load_model("cube".to_string(), Path::new("assets/test1.obj")).expect("Cannot load model");
        render_system.load_model("floor".to_string(), Path::new("assets/floor.obj")).expect("Cannot load model");
        //render_system.load_model("chalet".to_string(), Path::new("chalet.obj")).expect("Cannot load room");
        println!("Finished reading models");
    }


    pub fn render(&mut self, ecs: &ECS) {
        // Naive rendering right now. Do not order or anything.
        let objs: Vec<_> =  ecs.components.models
            .iter()
            .zip(ecs.components.transforms.iter())
            .filter(|(x, y)| x.is_some() && y.is_some())
            .map(|(x, y)| (x.as_ref().unwrap().value(),
            y.as_ref().unwrap().value())).collect();

        self.renderer.render(&ecs.camera, objs);
    }
}

// just an example to make some object move
pub struct DummySystem {
}

impl DummySystem {

    pub fn do_dumb_thing(&mut self, dt: Duration, ecs: &mut ECS) {
        let dt = dt_as_secs(dt);

        for (i, transform) in ecs.components.transforms.iter_mut()
            .enumerate()
            .filter(|(_, x)| x.is_some()) {

            match (*ecs.components.dummies).get_mut(i) {
                Some(dummy) => {
                    if let(Some(dummy)) = dummy {
                        let transform = transform.as_mut().unwrap();
                        transform.value_mut().scale.x += dt * dummy.value().speed;
                    }
                },
                None => {},
            }
            
        }
                
    }

}

