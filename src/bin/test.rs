use twgraph::net::Server;
use std::error::Error;
use tokio::prelude::*;

fn main() -> Result<(), Box<Error>> {
    
    let (server, _, _) = Server::connect(8080, 2)?;

    tokio::run(server.map_err(|err| println!("Error happened: {:?}", err)));
    Ok(())
}

