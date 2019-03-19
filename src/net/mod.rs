use log::error;
use std::net::SocketAddr;
use std::thread;
use bytes::{BytesMut, Bytes};
use crate::ecs::ECS;
use tokio::prelude::*;
mod server;
pub mod protocol;

use crate::sync::SharedDeque;

pub enum NetworkError {
    
}

pub use server::start_serving;

/// The network system is the ECS system that will be called in the main loop.
/// it should provide events and allow to send messages.
pub struct NetworkSystem {
    from_clients: SharedDeque<(BytesMut, SocketAddr)>,
    to_clients: std::sync::mpsc::Sender<(Bytes, SocketAddr)>,

    my_clients: Vec<SocketAddr>,
}


impl NetworkSystem {

    pub fn new(port: usize,
               max_clients: usize) -> Self {
        let (from_clients, to_clients) = start_serving(port).unwrap();

        let my_clients = vec![];

        Self {
            //server,
            to_clients,
            from_clients,
            my_clients,
        }
    }


    /// Will get the latest events that were sent to the server
    /// For example, player commands and so on.
    pub fn poll_events(&mut self) {

        let events = self.from_clients.drain();

        for (ev, client) in events {

            let mut found = false;
            for c in self.my_clients.iter() {
                if *c == client {
                    found = true;
                    break;
                }
            }

            if !found {
                self.my_clients.push(client);
            }
            println!("Network system received {:?}", ev);
        }   
    }


    /// This will send the current state to all clients.
    pub fn send_state(&mut self, ecs: &mut ECS) {

        let state = Bytes::from("hello");
        for c in self.my_clients.iter() {
            self.to_clients.send((state.clone(), *c));
        }
    }
}
