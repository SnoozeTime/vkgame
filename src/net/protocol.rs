use std::net::SocketAddr;
use serde_derive::{Serialize, Deserialize};
use rmp_serde::Serializer;
use serde::{Serialize, Deserialize};
use bytes::{BytesMut, Bytes};

#[derive(Debug, Clone, Copy)]
pub struct NetMessage {
    pub target: SocketAddr,
    pub content: NetMessageContent,
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

// Here we define all the messages that travel around client and servers.
//
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NetMessageContent {
    // Client sends that to the server.
    ConnectionRequest,

    // Server answers by accept or refuse
    ConnectionAccepted,
    ConnectionRefused,
}


pub fn deserialize(bytes: Bytes) -> Result<NetMessageContent, rmp_serde::decode::Error> {
    rmp_serde::from_slice::<NetMessageContent>(&bytes.to_vec())
}

pub fn serialize(msg: NetMessageContent) -> Result<Bytes, rmp_serde::encode::Error> {
    let mut b = Vec::new();
    msg.serialize(&mut Serializer::new(&mut b))?;
    Ok(b.into())
}
