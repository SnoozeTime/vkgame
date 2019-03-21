use std::net::SocketAddr;
use serde_derive::{Serialize, Deserialize};
use rmp_serde::Serializer;
use serde::{Serialize, Deserialize};
use bytes::{BytesMut, Bytes};

#[derive(Debug, Clone)]
pub struct NetMessage {
    pub target: SocketAddr,
    pub content: Packet,
}

impl NetMessage {

    /// return the message ready to be sent. Consume the object.
    pub fn pack(self) -> Result<(Bytes, SocketAddr), rmp_serde::encode::Error> {
        Ok((serialize(self.content)?, self.target))
    }

    pub fn unpack(buf: Bytes, target: SocketAddr) -> Result<NetMessage, rmp_serde::decode::Error> {

        Ok(NetMessage {
            content: deserialize(buf)?,
            target,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub seq_number: u32,
    pub content: NetMessageContent,
}

// Here we define all the messages that travel around client and servers.
//
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessageContent {
    // -----------------------------------
    // NETWORK LOGIC LEVEL 
    // -----------------------------------
    // Client sends that to the server.
    ConnectionRequest,

    // Server answers by accept or refuse
    ConnectionAccepted,
    ConnectionRefused,

    // ----------------------------------
    // GAME LOGIC LEVEL
    // ----------------------------------
    
    
    // ----------------------------------
    // FOR DEBUGGING
    // ----------------------------------
    Text(String),
}


pub fn deserialize(bytes: Bytes) -> Result<Packet, rmp_serde::decode::Error> {
    rmp_serde::from_slice::<Packet>(&bytes.to_vec())
}

pub fn serialize(msg: Packet) -> Result<Bytes, rmp_serde::encode::Error> {
    let mut b = Vec::new();
    msg.serialize(&mut Serializer::new(&mut b))?;
    Ok(b.into())
}
