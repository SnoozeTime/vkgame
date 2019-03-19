use log::{debug, error};
use std::net::SocketAddr;
use std::thread;
use bytes::{BytesMut, Bytes};
use crate::ecs::ECS;
use tokio::prelude::*;
mod server;
mod client;
pub mod protocol;

use crate::sync::SharedDeque;

pub enum NetworkError {
    
}

pub use server::start_serving;
pub use client::start_connecting;

/// The network system is the ECS system that will be called in the main loop.
/// it should provide events and allow to send messages.
pub struct NetworkSystem {
    from_clients: SharedDeque<protocol::NetMessage>,
    to_clients: std::sync::mpsc::Sender<protocol::NetMessage>,

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

        for ev in events {

            let mut found = false;
            for c in self.my_clients.iter() {
                if *c == ev.target {
                    found = true;
                    break;
                }
            }

            if !found {
                self.my_clients.push(ev.target);
            }
            debug!("Network system received {:?}", ev);
        }   
    }


    /// This will send the current state to all clients.
    pub fn send_state(&mut self, ecs: &mut ECS) {

       for c in self.my_clients.iter() {
           let msg = protocol::NetMessage { 
               content: protocol::NetMessageContent::ConnectionRefused,
               target: *c
           };
            self.to_clients.send(msg);
       }
    }
}
