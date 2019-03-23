use log::{debug, info, trace};
use std::error::Error;
use std::thread;
use std::time::Duration;

use twgraph::ecs::components::*;
use twgraph::ecs::*;
use twgraph::net::snapshot::*;
const NB_TRY: u32 = 10;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    let mut empty_ecs = ECS::new();
    let e = empty_ecs.new_entity();
    empty_ecs.components.models.set(
        &e,
        ModelComponent {
            mesh_name: "h".to_string(),
            texture_name: "h".to_string(),
        },
    );
    let mut empty_ecs_2 = ECS::new_from_existing(&empty_ecs);
    empty_ecs_2.delete_entity(&e);

    let full_ecs = ECS::load("arena.json")?;

    let mut full_ecs_2 = ECS::load("arena.json")?;

    full_ecs_2.delete_entity(&Entity::new(0, 0));
    dbg!(compute_delta(&full_ecs_2, &full_ecs));
    Ok(())
}
