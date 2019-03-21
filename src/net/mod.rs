use log::{debug, error, info};
use std::net::SocketAddr;
use std::thread;
use bytes::{BytesMut, Bytes};
use crate::ecs::ECS;
use tokio::prelude::*;
use std::fmt;
use std::error::Error;

mod delta;
mod server;
mod client;
pub mod protocol;


use crate::sync::SharedDeque;

#[derive(Debug)]
pub enum NetworkError {
    CannotConnectToServer,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NetworkError::CannotConnectToServer => write!(f, "Cannot connect to game server")
        }

    }
}

impl Error for NetworkError {

    fn description(&self) -> &str {
        match *self {
            NetworkError::CannotConnectToServer => "Cannot connect to game server", 
        }
    }

}


pub use server::start_serving;
pub use client::ClientSystem;

/// The network system is the ECS system that will be called in the main loop.
/// it should provide events and allow to send messages.
pub struct NetworkSystem {
    from_clients: SharedDeque<protocol::NetMessage>,
    to_clients: std::sync::mpsc::Sender<protocol::NetMessage>,

    my_clients: Vec<SocketAddr>,
    max_clients: usize,
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
            max_clients,
        }
    }


    /// Will get the latest events that were sent to the server
    /// For example, player commands and so on.
    pub fn poll_events(&mut self) {

        let events = self.from_clients.drain();

        for ev in events {

            if let protocol::NetMessageContent::ConnectionRequest = ev.content {
                self.handle_connection_request(ev.target);
            }

            debug!("Network system received {:?}", ev);
        }   
    }


    /// This will send the current state to all clients.
    pub fn send_state(&mut self, ecs: &mut ECS) {

        for c in self.my_clients.iter() {
            let msg = protocol::NetMessage { 
                content: protocol::NetMessageContent::Text(String::from("Bonjour")),
                target: *c
            };
            self.to_clients.send(msg);
        }
    }

    /// This is called when a ConnectionRequest message is received
    /// It will reply with either connection accepted or connection refused
    /// and add the client to our map of clients.
    ///
    /// If a client is already in the map, it should reply connection
    /// accepted. The reason is that the connection acception message
    /// might have been lost so the client thinks it is still trying to connect
    fn handle_connection_request(&mut self, addr: SocketAddr) {

        info!("Handle new connection request from {}", addr);

        let to_send = {
            if self.has_client(addr) {
                info!("Client was already connected, resend ConnectionAccepted");
                protocol::NetMessageContent::ConnectionAccepted
            } else { 
                // in that case we need to find an empty slot. If available,
                // return connection accepted.
                if self.my_clients.len() < self.max_clients {
                    info!("Client connected");
                    self.my_clients.push(addr);
                    protocol::NetMessageContent::ConnectionAccepted
                } else {
                    info!("Too many clients connected, send ConnectionRefused");
                    protocol::NetMessageContent::ConnectionRefused
                }
            }
        };

        self.to_clients.send(protocol::NetMessage {
            target: addr,
            content: to_send,
        });
    }


    fn has_client(&self, addr: SocketAddr) -> bool {
        self.my_clients.iter().any(|&client| {
            client == addr
        })
    }
}
