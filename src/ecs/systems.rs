use winit::{WindowBuilder};
use vulkano::instance::Instance;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;

use std::sync::Arc;
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
        std::path::Path::new("src/image_img.png"),
        93, 93).unwrap();
        render_system.load_texture("white".to_string(),
        std::path::Path::new("src/white.png"),
        93, 93).unwrap();
    }

    fn init_models(render_system: &mut Renderer) {
        render_system.load_model("cube".to_string(), std::path::Path::new("cube.obj")).expect("Cannot load model");
    }


    pub fn render(&mut self, ecs: &ECS) {

        // Naive rendering right now. Do not order or anything.
        let objs: Vec<_> =  ecs.model_components
            .iter()
            .zip(ecs.transform_components.iter())
            .filter(|(x, y)| x.is_some() && y.is_some())
            .map(|(x, y)| (x.as_ref().unwrap().value(),
                           y.as_ref().unwrap().value())).collect();

        self.renderer.render(&ecs.camera, objs);
    }
}

