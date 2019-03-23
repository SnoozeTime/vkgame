use crate::ecs::ECS;
use bytes::{Bytes, BytesMut};
use log::{debug, error, info};
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::thread;
use tokio::prelude::*;

mod client;
pub mod protocol;
mod server;
pub mod snapshot;

use crate::sync::SharedDeque;

#[derive(Debug)]
pub enum NetworkError {
    CannotConnectToServer,
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NetworkError::CannotConnectToServer => write!(f, "Cannot connect to game server"),
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

pub use client::ClientSystem;
pub use server::NetworkSystem;
