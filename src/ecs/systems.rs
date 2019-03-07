use winit::{WindowBuilder, Window};
use imgui::{FontGlyphRange, ImFontConfig, ImGui};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano::device::{Device, Queue};
use vulkano_win::VkSurfaceBuild;
use vulkano_win;
use cgmath::{Angle,Rad};

use std::sync::Arc;
use std::time::Duration;

use crate::renderer::Renderer;
use crate::time::dt_as_secs;
use crate::resource::Resources;
use crate::ui::Gui;

use super::{Entity, ECS};

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
        let renderer = Renderer::new(&mut imgui, &instance, surface.clone()).unwrap();

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

    pub fn get_device(&self) -> Arc<Device> {
        self.renderer.device.clone()
    }

    pub fn get_queue(&self) -> Arc<Queue> {
        self.renderer.queue.clone()
    }

    pub fn get_surface(&self) -> Arc<Surface<winit::Window>> {
        self.renderer.surface.clone()
    }


    pub fn render(&mut self,
                  resources: &Resources,
                  ecs: &mut ECS,
                  dt: Duration,
                  gui: &mut Gui)
    {
        let dt = dt_as_secs(dt) as f32;

        // TODO SHOULD NOT BE DONE HERE.
        // It needs to be in its own class and we can pass the ui here maybe.
        let window = self.surface.window();
        imgui_winit_support::update_mouse_cursor(&self.imgui,
                                                 &window);
        let frame_size = imgui_winit_support::get_frame_size(&window, 
                                                             self.hidpi_factor).unwrap();
        let ui = self.imgui.frame(frame_size, dt);
        if !gui.run_ui(&ui, ecs) {
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


        self.renderer.render(resources, ui, &mut ecs.camera, lights, objs);
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

    pub fn handle_events(&mut self, events: &Vec<crate::event::Event>) {

        self.renderer.handle_events(events);
    }

    pub fn pick_object(&mut self, x: f64, y: f64, ecs: &ECS, resources: &Resources) -> Option<Entity> {
        self.renderer.pick_object(x, y, ecs, resources)
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
        let dt = dt_as_secs(dt) as f32;

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

