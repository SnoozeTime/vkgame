use log::{debug, info, trace};
use std::error::Error;
use std::thread;
use std::time::Duration;

use twgraph::ecs::components::*;
use twgraph::ecs::*;
use twgraph::net::snapshot::*;

fn main() -> Result<(), Box<Error>> {
    env_logger::init();

    Ok(())
}
