use serde_derive::{Serialize, Deserialize};

// Here we define all the messages that travel around client and servers.
//
#[derive(Debug, Clone, Serialize, Deserialize)]
enum NetMessage {
    // Client sends that to the server.
    ConnectionRequest,

    // Server answers by accept or refuse
    ConnectionAccepted,
    ConnectionRefused,
}
