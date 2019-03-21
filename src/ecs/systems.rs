use winit::{WindowBuilder, Window, Event, WindowEvent};
use imgui::{FontGlyphRange, ImFontConfig, ImGui};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano::device::{Device, Queue};
use vulkano_win::VkSurfaceBuild;
use vulkano_win;
use cgmath::{Angle,Rad};

use std::sync::Arc;
use std::time::Duration;
use log::{trace, error};

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
//           .with_dimensions((600, 600).into())
//           .with_resizable(false)
            .build_vk_surface(&events_loop, instance.clone())
            .expect("Cannot create vk_surface");

        let window = surface.window();
        window.hide_cursor(true);
        //window.set_maximized(true);


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

    /// FIXME should be called by scene at creation
    pub fn grab_cursor(&mut self, should_grab: bool) {
        if let Err(err) = self.surface.window().grab_cursor(should_grab) {
            error!("{:?}", err);
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
        

        // That's a lot of memory allocation here. FIXME
        let live_entities = ecs.nb_entities();
        let mut lights = Vec::new();
        let mut objs = Vec::new();

        for entity in &live_entities {
            let maybe_l = ecs.components.lights.get(entity);
            let maybe_t = ecs.components.transforms.get(entity);
            let maybe_m = ecs.components.models.get(entity);

            match (maybe_l, maybe_m, maybe_t) {
                (Some(l), _, Some(t)) => lights.push((l,t)),
                (_, Some(m), Some(t)) => objs.push((m, t)),
                _ => {}
            }
        }

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

        if let Event::WindowEvent { 
            event: WindowEvent::Resized(x),
            ..} = ev {
            trace!("Got resize ev: {:?}", x);
        }
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

pub struct GravitySystem {
}

impl GravitySystem {
    pub fn new() -> Self {
        GravitySystem {}
    }

    pub fn do_gravity(&mut self, dt: Duration, ecs: &mut ECS) {
        let dt = dt_as_secs(dt) as f32;

        let live_entities = ecs.nb_entities();
//        let mut physics_objects = Vec::new();

        // Get all transform + gravity entities
        for entity in &live_entities {
            let maybe_g = ecs.components.rigid_bodies.get_mut(entity);
            let maybe_t = ecs.components.transforms.get_mut(entity);

            match (maybe_g, maybe_t) {
                (Some(g),Some(t)) =>
                    if t.position.y >= 0f32 {
                        t.position.y = t.position.y - dt * 0.5f32;
                    } else {
                        t.position.y = 0f32;
                    },
                _ => {}
            }
        }
    }
}
