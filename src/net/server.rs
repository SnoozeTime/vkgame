use bytes::{Bytes, BytesMut};
use futures::sync::mpsc as futmpsc;
use log::{debug, error, info, trace, warn};
use std::io;
use std::net::SocketAddr;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::stream::{SplitSink, SplitStream};
use tokio::prelude::*;
use tokio_codec::BytesCodec;

use super::protocol;
use super::protocol::Packet;
use std::thread;

use std::sync::mpsc as stdmpsc;

use super::NetworkError;
use crate::ecs::ECS;
use crate::sync::SharedDeque;

pub fn start_serving(
    port: usize,
) -> Result<
    (
        SharedDeque<protocol::NetMessage>,
        stdmpsc::Sender<protocol::NetMessage>,
    ),
    Box<std::error::Error>,
> {
    info!("Start serving on {}", port);
    // interfaces
    let net_to_game = SharedDeque::new(1024);
    let mut net_to_game_clone = net_to_game.clone();
    let (int_tx, int_rx) = futmpsc::channel(1024);
    let (tx, rx) = stdmpsc::channel();
    let int_rx = int_rx.map_err(|_| panic!("Error not possible on rx"));

    thread::spawn(move || read_channel(int_tx, rx));

    let async_stuff = connect(port, Box::new(int_rx))?;
    thread::spawn(move || {
        tokio::run(
            async_stuff
                .for_each(move |(buf, client)| {
                    match protocol::NetMessage::unpack(buf.into(), client) {
                        Ok(unpacked) => net_to_game_clone.push(unpacked),
                        Err(e) => {
                            error!(
                                "Received malformed message from {}, error = {:?}",
                                client, e
                            );
                        }
                    }

                    Ok(())
                })
                .map_err(|e| error!("{:?}", e)),
        );
    });

    Ok((net_to_game, tx))
}

/// Will create the futures that will run in tokio runtime.
fn connect(
    port: usize,
    game_to_net: Box<Stream<Item = (Bytes, SocketAddr), Error = io::Error> + Send>,
) -> Result<
    Box<Stream<Item = (BytesMut, SocketAddr), Error = io::Error> + Send>,
    Box<std::error::Error>,
> {
    let addr = format!("127.0.0.1:{}", port).parse()?;
    let socket = UdpSocket::bind(&addr)?;

    let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

    // All bytes from `game_to_net` will go to the `addr` specified in our
    // argument list. Like with TCP this is spawned concurrently
    let forward = game_to_net.forward(sink).then(|result| {
        if let Err(e) = result {
            error!("failed to write to socket: {}", e)
        }
        Ok(())
    });

    let all_futs = Box::new(
        future::lazy(|| {
            tokio::spawn(forward);
            future::ok(stream)
        })
        .flatten_stream(),
    );

    Ok(all_futs)
}

fn read_channel(
    mut tx: futmpsc::Sender<(Bytes, SocketAddr)>,
    rx: stdmpsc::Receiver<protocol::NetMessage>,
) {
    loop {
        let d = rx.recv().unwrap();

        // if cannot serialize here, we have a problem...
        let packed = d
            .pack()
            .map_err(|e| {
                error!("Error when unpacking in `read_channel` = {:?}", e);
                e
            })
            .unwrap();

        tx = match tx.send(packed).wait() {
            Ok(tx) => tx,
            Err(e) => {
                error!("Error in read_channel = {:?}", e);
                break;
            }
        }
    }
}

// State of each clients
#[derive(Debug, Clone)]
struct Client {
    // IP/Port
    addr: SocketAddr,

    // Index in the snapshot circular buffer
    // None is hasn't received information yet
    last_state: Option<usize>,

    // Incremented nb that is sent in the packet
    last_seq_number: u32,
}

/// The network system is the ECS system that will be called in the main loop.
/// it should provide events and allow to send messages.
pub struct NetworkSystem {
    from_clients: SharedDeque<protocol::NetMessage>,
    to_clients: std::sync::mpsc::Sender<protocol::NetMessage>,

    my_clients: Vec<Client>,
    max_clients: usize,
}

impl NetworkSystem {
    pub fn new(port: usize, max_clients: usize) -> Self {
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
            if let protocol::NetMessageContent::ConnectionRequest = ev.content.content {
                self.handle_connection_request(ev.target);
            }

            debug!("Network system received {:?}", ev);
        }
    }

    /// This will send the current state to all clients.
    pub fn send_state(&mut self, ecs: &mut ECS) {
        for i in 0..self.my_clients.len() {
            let msg = protocol::NetMessageContent::Text(String::from("Bonjour"));
            self.send_to_client(i, msg);
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

        let (to_send, client_id) = {
            if let Some(id) = self.get_client_id(addr) {
                info!("Client was already connected, resend ConnectionAccepted");
                (protocol::NetMessageContent::ConnectionAccepted, Some(id))
            } else {
                // in that case we need to find an empty slot. If available,
                // return connection accepted.
                if self.my_clients.len() < self.max_clients {
                    info!("Client connected");
                    self.my_clients.push(Client {
                        addr,
                        last_seq_number: 0,
                        last_state: None,
                    });
                    (
                        protocol::NetMessageContent::ConnectionAccepted,
                        Some(self.my_clients.len() - 1),
                    )
                } else {
                    info!("Too many clients connected, send ConnectionRefused");
                    (protocol::NetMessageContent::ConnectionRefused, None)
                }
            }
        };

        if let Some(id) = client_id {
            self.send_to_client(id, to_send);
        } else {
            // ConnectionRefused is sent to parties that are not client yet.
            self.to_clients.send(protocol::NetMessage {
                target: addr,
                content: Packet {
                    content: to_send,
                    seq_number: 0,
                    last_known_state: None,
                },
            });
        }
    }

    /// Should be used to send a message to a client. Will increase a sequence number.
    fn send_to_client(&mut self, client_id: usize, msg: protocol::NetMessageContent) {
        let to_send = protocol::NetMessage {
            target: self.my_clients[client_id].addr,
            content: Packet {
                content: msg,
                seq_number: self.my_clients[client_id].last_seq_number,
                last_known_state: None, // doesn't matter on server->client
            },
        };

        if let Err(e) = self.to_clients.send(to_send) {
            error!("Error in send_to_client = {:?}", e);
        } else {
            self.my_clients[client_id].last_seq_number += 1;
        }
    }

    fn has_client(&self, addr: SocketAddr) -> bool {
        self.my_clients.iter().any(|client| client.addr == addr)
    }

    fn get_client_id(&self, addr: SocketAddr) -> Option<usize> {
        self.my_clients
            .iter()
            .enumerate()
            .find(|(_, client)| client.addr == addr)
            .map(|t| t.0)
    }
}
