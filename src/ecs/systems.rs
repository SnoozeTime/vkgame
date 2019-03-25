use cgmath::{Angle, InnerSpace, Rad, Vector3};
use imgui::{FontGlyphRange, ImFontConfig, ImGui};
use log::debug;
use vulkano::device::{Device, Queue};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win;
use vulkano_win::VkSurfaceBuild;
use winit::{Window, WindowBuilder, WindowEvent};

use log::{error, trace};
use std::sync::Arc;
use std::time::Duration;

use super::{Entity, ECS};
use crate::camera::CameraDirection;
use crate::event::Event;
use crate::renderer::Renderer;
use crate::resource::Resources;
use crate::scene::ClientCommand;
use crate::time::dt_as_secs;
use crate::ui::Gui;

use std::collections::{HashMap, HashSet};

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

    pub fn render(&mut self, resources: &Resources, ecs: &mut ECS, dt: Duration, gui: &mut Gui) {
        let dt = dt_as_secs(dt) as f32;

        // TODO SHOULD NOT BE DONE HERE.
        // It needs to be in its own class and we can pass the ui here maybe.
        let window = self.surface.window();
        imgui_winit_support::update_mouse_cursor(&self.imgui, &window);
        let frame_size = imgui_winit_support::get_frame_size(&window, self.hidpi_factor).unwrap();
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
                (Some(l), _, Some(t)) => lights.push((l, t)),
                (_, Some(m), Some(t)) => objs.push((m, t)),
                _ => {}
            }
        }

        self.renderer
            .render(resources, ui, &mut ecs.camera, lights, objs);
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

        if let winit::Event::WindowEvent {
            event: WindowEvent::Resized(x),
            ..
        } = ev
        {
            trace!("Got resize ev: {:?}", x);
        }
    }

    pub fn handle_events(&mut self, events: &Vec<Event>) {
        self.renderer.handle_events(events);
    }

    pub fn pick_object(
        &mut self,
        x: f64,
        y: f64,
        ecs: &ECS,
        resources: &Resources,
    ) -> Option<Entity> {
        self.renderer.pick_object(x, y, ecs, resources)
    }
}

// just an example to make some object move
pub struct DummySystem {
    angle: f32,
}

impl DummySystem {
    pub fn new() -> Self {
        DummySystem { angle: 0.0 }
    }

    pub fn do_dumb_thing(&mut self, dt: Duration, ecs: &mut ECS) {
        let dt = dt_as_secs(dt) as f32;

        self.angle += dt;
        for (i, transform) in ecs
            .components
            .transforms
            .iter_mut()
            .enumerate()
            .filter(|(_, x)| x.is_some())
        {
            match (*ecs.components.dummies).get_mut(i) {
                Some(dummy) => {
                    if let Some(dummy) = dummy {
                        let transform = transform.as_mut().unwrap().value_mut();
                        transform.position.x = 5.0 * Rad(self.angle * dummy.value().speed).cos();
                        transform.position.z = 5.0 * Rad(self.angle * dummy.value().speed).sin();
                        //transform.value_mut().scale.x += dt * dummy.value().speed;
                    }
                }
                None => {}
            }
        }
    }
}

/// In charge of updating players positions and so on from the events (network + physics)
pub struct PlayerSystem {
    /// Store the commands that should be applied to players at each frame.
    ///
    /// This also play a bit the role of a frame limiter. For example if a player sends
    /// too many move packets during one frame, we are going to use only one here.
    commands_per_players: HashMap<Entity, HashSet<CameraDirection>>,

    world_up: Vector3<f32>,
}

impl PlayerSystem {
    pub fn new() -> Self {
        PlayerSystem {
            commands_per_players: HashMap::new(),
            world_up: Vector3::new(0.0, 1.0, 0.0),
        }
    }

    /// Update commands to apply each player. Also, update their look_at vector
    pub fn handle_network_events(&mut self, ecs: &mut ECS, events: &Vec<(Entity, Event)>) {
        for v in self.commands_per_players.values_mut() {
            v.clear();
        }

        for (entity, event) in events {
            if let None = ecs.components.players.get(&entity) {
                debug!(
                    "Got an event {:?} for entity {:?} that is not a player",
                    event, entity
                );
                continue;
            }

            match event {
                // Look at update will just update the direction where the player
                // is looking at.
                Event::ClientEvent(ClientCommand::LookAt(direction)) => {
                    let mut comp = ecs.components.players.get_mut(&entity).unwrap();
                    comp.look_at =
                        Vector3::new(direction[0], direction[1], direction[2]).normalize();
                    comp.right = comp.look_at.cross(self.world_up).normalize();
                    comp.up = comp.right.cross(comp.look_at).normalize();
                }

                // Move update will add an element to the commands_per_player
                // hashset. These will be processed in the update functions.
                // Basically, handle_network_events should be called before
                // the update function.
                Event::ClientEvent(ClientCommand::Move(direction)) => {
                    if !self.commands_per_players.contains_key(&entity) {
                        self.commands_per_players
                            .insert(entity.clone(), HashSet::new());
                    }

                    self.commands_per_players
                        .get_mut(&entity)
                        .unwrap()
                        .insert(*direction);
                }
                _ => (),
            }
        }
    }

    pub fn update(&self, dt: Duration, ecs: &mut ECS) {
        for (entity, events) in self.commands_per_players.iter() {
            let mut transform = ecs
                .components
                .transforms
                .get_mut(&entity)
                .expect("Player does not have a transform, but it should...");
            let player = ecs.components.players.get(&entity).unwrap();

            let dt_as_secs = dt_as_secs(dt);
            let proj_front = player.look_at - (player.look_at.dot(self.world_up)) * self.world_up;
            let proj_right = player.right - (player.right.dot(self.world_up)) * self.world_up;
            for event in events {
                match *event {
                    CameraDirection::Forward => {
                        // TODO replace 10.0 by player speed
                        transform.position += 10.0 * dt_as_secs as f32 * proj_front;
                    }
                    CameraDirection::Backward => {
                        transform.position -= 10.0 * dt_as_secs as f32 * proj_front;
                    }
                    CameraDirection::Left => {
                        transform.position -= 10.0 * dt_as_secs as f32 * proj_right;
                    }
                    CameraDirection::Right => {
                        transform.position += 10.0 * dt_as_secs as f32 * proj_right;
                    }
                    _ => {}
                }
            }
        }
    }
}
