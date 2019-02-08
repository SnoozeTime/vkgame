use winit::{WindowBuilder};
use vulkano::instance::Instance;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;

use std::sync::Arc;
use std::path::Path;
use crate::renderer::Renderer;

use super::ECS;

pub struct RenderingSystem<'a> {
    renderer: Renderer<'a>,    
}

impl<'a> RenderingSystem<'a> {

    pub fn new(instance: &'a Arc<Instance>, events_loop: &winit::EventsLoop) -> Self {
        // Get the surface and window. Window is from winit library
        let surface = WindowBuilder::new()
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
    //render_system.load_texture("chalet".to_string(),
        //Path::new("chalet.jpg"),
       // 4096, 4096).unwrap();
    }

    fn init_models(render_system: &mut Renderer) {
        println!("Init models!");
        render_system.load_model("cube".to_string(), Path::new("assets/cube.obj")).expect("Cannot load model");
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

