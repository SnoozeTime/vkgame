use winit::{WindowBuilder, Window};
use imgui::{FontGlyphRange, ImFontConfig, ImGui, Ui};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;
use cgmath::{Angle,Rad};

use std::sync::Arc;
use std::path::Path;
use std::time::Duration;

use crate::renderer::Renderer;
use crate::time::dt_as_secs;

use super::ECS;

pub struct RenderingSystem<'a> {
    renderer: Renderer<'a>,    
    imgui: ImGui,

    surface: Arc<Surface<Window>>,
    hidpi_factor: f64,
}

impl<'a> RenderingSystem<'a> {

    pub fn new(instance: &'a Arc<Instance>, events_loop: &winit::EventsLoop) -> Self {
        // Get the surface and window. Window is from winit library
        let surface = WindowBuilder::new()
//            .with_dimensions((600, 600).into())
//           .with_resizable(false)
            .build_vk_surface(&events_loop, instance.clone())
            .expect("Cannot create vk_surface");

        let window = surface.window();
        window.grab_cursor(true).unwrap();
        window.hide_cursor(true);


        // Set up ImGUI
        // -----------------------------------------------------
        let mut imgui = ImGui::init();
        let hidpi_factor = window.get_hidpi_factor().round();

        let font_size = (13.0 * hidpi_factor) as f32;

        imgui.fonts().add_default_font_with_config(
            ImFontConfig::new()
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size),
            );

        imgui.fonts().add_font_with_config(
            include_bytes!("mplus-1p-regular.ttf"),
            ImFontConfig::new()
            .merge_mode(true)
            .oversample_h(1)
            .pixel_snap_h(true)
            .size_pixels(font_size)
            .rasterizer_multiply(1.75),
            &FontGlyphRange::japanese(),
            );

        imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);


        imgui_winit_support::configure_keys(&mut imgui);


        // TODO error handling
        let mut renderer = Renderer::new(&mut imgui, &instance, surface.clone()).unwrap();
        Self::init_textures(&mut renderer);
        Self::init_models(&mut renderer);

        RenderingSystem {
            renderer,
            imgui,
            surface,
            hidpi_factor,
        }
    }

    pub fn dimensions(&self) -> [u32; 2] {
        self.renderer.dimensions()
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
        render_system.load_model("building".to_string(), Path::new("assets/models/arena.obj")).expect("Cannot load model");

        //render_system.load_model("chalet".to_string(), Path::new("chalet.obj")).expect("Cannot load room");
        println!("Finished reading models");
    }


    pub fn render<F>(&mut self,
                  ecs: &mut ECS,
                  dt: Duration,
                  mut run_ui: F)
        where F: FnMut(&Ui, &mut ECS) -> bool,
    {
        let dt = dt_as_secs(dt);

        // TODO SHOULD NOT BE DONE HERE.
        // It needs to be in its own class and we can pass the ui here maybe.
        let window = self.surface.window();
        imgui_winit_support::update_mouse_cursor(&self.imgui,
                                                 &window);
        let frame_size = imgui_winit_support::get_frame_size(&window, 
                                                             self.hidpi_factor).unwrap();
        let ui = self.imgui.frame(frame_size, dt);
        if !run_ui(&ui, ecs) {
            panic!("wuuuuut");
        }

        // Get the lights.
        let lights: Vec<_> =  ecs.components.lights
            .iter()
            .zip(ecs.components.transforms.iter())
            .filter(|(x, y)| x.is_some() && y.is_some())
            .map(|(x, y)| (x.as_ref().unwrap().value(),
            y.as_ref().unwrap().value())).collect();

        // Naive rendering right now. Do not order or anything.
        let objs: Vec<_> =  ecs.components.models
            .iter()
            .zip(ecs.components.transforms.iter())
            .filter(|(x, y)| x.is_some() && y.is_some())
            .map(|(x, y)| (x.as_ref().unwrap().value(),
            y.as_ref().unwrap().value())).collect();


        self.renderer.render(ui, &mut ecs.camera, lights, objs);
    } 

    /// Should be passed in the event polling
    pub fn handle_event(&mut self, ev: &winit::Event) {
        let window = self.surface.window();
            imgui_winit_support::handle_event(
                &mut self.imgui,
                ev,
                window.get_hidpi_factor(),
                self.hidpi_factor,
                );
    }
}

// just an example to make some object move
pub struct DummySystem {
    angle: f32,
}


impl DummySystem {

    pub fn new() -> Self {
        DummySystem {
            angle: 0.0,
        }
    }

    pub fn do_dumb_thing(&mut self, dt: Duration, ecs: &mut ECS) {
        let dt = dt_as_secs(dt);

        self.angle += dt;
        for (i, transform) in ecs.components.transforms.iter_mut()
            .enumerate()
                .filter(|(_, x)| x.is_some()) {

                    match (*ecs.components.dummies).get_mut(i) {
                        Some(dummy) => {
                            if let Some(dummy) = dummy {
                                let transform = transform.as_mut().unwrap().value_mut();
                                transform.position.x = 5.0 * Rad(self.angle*dummy.value().speed).cos();
                                transform.position.z = 5.0 * Rad(self.angle*dummy.value().speed).sin();
                                //transform.value_mut().scale.x += dt * dummy.value().speed;
                            }
                        },
                        None => {},
                    }

                }

    }

}

