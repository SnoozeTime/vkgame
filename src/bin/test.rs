use twgraph::net::protocol::*;
use std::error::Error;
use log::{debug, info, trace};
use std::thread;
use std::time::Duration;
use twgraph::net::ClientSystem;


const NB_TRY: u32 = 10;

fn main() -> Result<(), Box<Error>> { 

    env_logger::init();
    let mut client = ClientSystem::connect("127.0.0.1:8080".parse()?)?;
    info!("Connected to server");
    loop {

        client.poll_events();
        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

