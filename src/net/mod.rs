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


pub use server::NetworkSystem;
pub use client::ClientSystem;

