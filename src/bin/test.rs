use log::{debug, info, trace};
use std::error::Error;
use std::thread;
use std::time::Duration;

use twgraph::ecs::*;
use twgraph::net::snapshot::*;
const NB_TRY: u32 = 10;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    let empty_ecs = ECS::new();
    let full_ecs = ECS::load("arena.json")?;

    dbg!(compute_delta(&full_ecs, &full_ecs));
    Ok(())
}
