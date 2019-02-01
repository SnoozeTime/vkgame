use winit::{WindowBuilder, Window};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;

use std::sync::Arc;

use crate::renderer::{Renderer};

pub struct RenderingSystem<'a> {
    surface: Arc<Surface<Window>>,
    renderer: Renderer<'a>,    
}

impl<'a> RenderingSystem<'a> {

    fn new(instance: &'a Arc<Instance>, events_loop: &mut winit::EventsLoop) -> Self {
        // Get the surface and window. Window is from winit library
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .expect("Cannot create vk_surface");

        // TODO error handling
        let mut renderer = Renderer::new(&instance, surface.clone()).unwrap();

        RenderingSystem {
            surface,
            renderer,
        }
    }

}
