use log::error;
use futures::sync::mpsc;
use bytes::Bytes;
use crate::ecs::ECS;
mod server;
mod protocol;

pub enum NetworkError {
    
}

pub use server::Server;

/// The network system is the ECS system that will be called in the main loop.
/// it should provide events and allow to send messages.
pub struct NetworkSystem {

    server: Server,

    from_clients: mpsc::UnboundedReceiver<Bytes>,
    to_clients: mpsc::UnboundedSender<Bytes>,
}


impl NetworkSystem {

    pub fn new(port: usize,
               max_clients: usize) -> Self {
    
        // If error here, just crash the server and display log.
        let server = Server::connect(port, max_clients);
        if let Err(err) = server {
            error!("Cannot create server = {:?}", err);
            panic!("Cannot create server, that should not happen. Please check the command line arguments");
        }

        let (server, to_clients, from_clients) = server.unwrap();

        Self {
            server,
            to_clients,
            from_clients,
        }
    }


    /// Will get the latest events that were sent to the server
    /// For example, player commands and so on.
    pub fn poll_events(&mut self) {

    }


    /// This will send the current state to all clients.
    pub fn send_state(&mut self, ecs: &mut ECS) {

    }
}
