use super::snapshot::DeltaSnapshot;
use crate::camera::CameraDirection;
use crate::scene::ClientCommand;
use bytes::{Bytes, BytesMut};
use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct NetMessage {
    pub target: SocketAddr,
    pub content: Packet,
}

impl NetMessage {
    /// return the message ready to be sent. Consume the object.
    pub fn pack(self) -> Result<(Bytes, SocketAddr), serde_json::error::Error> {
        Ok((serialize(self.content)?, self.target))
    }

    pub fn unpack(buf: Bytes, target: SocketAddr) -> Result<NetMessage, serde_json::error::Error> {
        Ok(NetMessage {
            content: deserialize(buf)?,
            target,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    pub seq_number: u32,
    // Only matter on client>erver side. Should we remove from here and put in NetMessageContent
    // instead?
    pub last_known_state: Option<u8>,
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

    Ping,

    // ----------------------------------
    // GAME LOGIC LEVEL
    // ----------------------------------
    // contain the server state.
    Delta(DeltaSnapshotInfo),

    // Command from the client.
    Command(ClientCommand),

    // ----------------------------------
    // FOR DEBUGGING
    // ----------------------------------
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSnapshotInfo {
    pub old_state: Option<u8>,
    pub new_state: u8,
    pub delta: DeltaSnapshot,
}

pub fn deserialize(bytes: Bytes) -> Result<Packet, serde_json::error::Error> {
    serde_json::from_slice::<Packet>(&bytes.to_vec())
}

pub fn serialize(msg: Packet) -> Result<Bytes, serde_json::error::Error> {
    //let mut b = Vec::new();
    let b = serde_json::to_vec(&msg)?;
    //msg.serialize(&mut Serializer::new(&mut b))?;
    Ok(b.into())
}
