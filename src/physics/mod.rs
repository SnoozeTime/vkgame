use crate::ecs::{Entity, ECS};
use std::time::Duration;
use crate::time::dt_as_secs;
use cgmath::Vector3;

use log::debug;

pub struct PhysicsSystem {
}

impl PhysicsSystem {
    pub fn new() -> Self {
        PhysicsSystem {}
    }

    pub fn on_update(&mut self, dt: Duration, ecs: &mut ECS) {
        // Physics system seems to follow this pattern
        // 1. Do broadphase to locate potential interacting pairs
        // -> Throw in algorithm here later

        // 2. Do collision detection on everything
        // -> Given pairs of potential colliding objects, determine if they collide during dt

        // 3. Resolution - apply forces, move transforms etc to keep physics in check
        self.apply_motion(dt, ecs);

    }

    fn apply_motion(&mut self, dt: Duration, ecs: &mut ECS) {
        let dt = dt_as_secs(dt) as f32;

        // Get all entities
        let live_entities = ecs.nb_entities();


        for entity in &live_entities {
            let maybe_g = ecs.components.rigid_bodies.get_mut(entity);
            let maybe_t = ecs.components.transforms.get_mut(entity);

            let up = Vector3::new(0.0, 1.0, 0.0);

            // Simple movement downwards from now
            match (maybe_g, maybe_t) {
                (Some(g),Some(t)) =>
                    if t.position.y > 0f32 + (t.scale.y / 2f32) {
                        g.velocity += -9.82f32 * up * dt;
                        t.position += dt * g.velocity;
                        //println!("{:?}", t.position.y);
                        debug!("velocity -> {:?}", g.velocity);
                        debug!("position -> {:?}", t.position);
                    } else {
                        t.position.y = 0f32;
                    },
                _ => {}
            }
        }

    }
}