use serde_derive::{Serialize, Deserialize};
use rmp_serde::Serializer;
use serde::{Serialize, Deserialize};
use bytes::{BytesMut, Bytes};

// Here we define all the messages that travel around client and servers.
//
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetMessage {
    // Client sends that to the server.
    ConnectionRequest,

    // Server answers by accept or refuse
    ConnectionAccepted,
    ConnectionRefused,
}


pub fn deserialize(bytes: Bytes) -> Result<NetMessage, rmp_serde::decode::Error> {
    rmp_serde::from_slice::<NetMessage>(&bytes.to_vec())
}

pub fn serialize(msg: NetMessage) -> Result<Bytes, rmp_serde::encode::Error> {
    let mut b = Vec::new();
    msg.serialize(&mut Serializer::new(&mut b))?;
    Ok(b.into())
}
