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
use super::protocol::{DeltaSnapshotInfo, Packet};
use std::thread;

use std::sync::mpsc as stdmpsc;

use super::NetworkError;
use crate::camera::CameraDirection;
use crate::collections::OptionArray;
use crate::ecs::{
    components::{ModelComponent, TransformComponent},
    Entity, ECS,
};
use crate::event::{Event, GameEvent};
use crate::net::snapshot::{DeltaSnapshot, SnapshotError, Snapshotter};
use crate::sync::SharedDeque;
use cgmath::Vector3;

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
    last_state: Option<u8>,

    // Incremented nb that is sent in the packet
    last_rec_seq_number: u32,
    last_sent_seq_number: u32,

    // The entity in the server ECS associated to this client
    entity: Option<Entity>,
}

/// The network system is the ECS system that will be called in the main loop.
/// it should provide events and allow to send messages.
pub struct NetworkSystem {
    from_clients: SharedDeque<protocol::NetMessage>,
    to_clients: std::sync::mpsc::Sender<protocol::NetMessage>,

    my_clients: OptionArray<Client>,

    snapshotter: Snapshotter,
}

impl NetworkSystem {
    pub fn new(port: usize, max_clients: usize) -> Self {
        let (from_clients, to_clients) = start_serving(port).unwrap();

        let my_clients = OptionArray::new(max_clients);

        Self {
            //server,
            to_clients,
            from_clients,
            my_clients,
            snapshotter: Snapshotter::new(60),
        }
    }

    /// Will get the latest events that were sent to the server
    /// For example, player commands and so on.
    ///
    /// Returns a list of events generated by a player
    pub fn poll_events(&mut self, ecs: &mut ECS) -> Vec<(Entity, Event)> {
        let events = self.from_clients.drain();

        let mut game_events = vec![];

        for ev in events {
            trace!("Network system received {:?}", ev);
            if let protocol::NetMessageContent::ConnectionRequest = ev.content.content {
                self.handle_connection_request(ev.target, ecs);
            } else {
                // if the client is known, send OK, else send connection refused. Update
                // the last known state so that we send the correct thing in snapshots.
                if let Some(index) = self.get_client_id(ev.target) {
                    let client = self.my_clients.get_mut(index).unwrap();

                    // Discard out of order.
                    if client.last_rec_seq_number >= ev.content.seq_number {
                        error!("Receive packet out of order for {}: last_rec_seq_number {} >= packet.seq_number {}", ev.target, client.last_rec_seq_number, ev.content.seq_number);
                    } else {
                        client.last_state = ev.content.last_known_state;
                        client.last_rec_seq_number = ev.content.seq_number;

                        // Now convert the message as an event that will be processed by the
                        // engine (physics,... and so on).
                        if let Some(ev) = NetworkSystem::handle_client_message(&client, ev.content)
                        {
                            game_events.push((client.entity.unwrap().clone(), ev));
                        }
                    }
                } else {

                }
            }
        }

        game_events
    }

    /// This will send the current state to all clients.
    pub fn send_state(&mut self, ecs: &mut ECS) {
        // First take a snapshot.
        self.snapshotter.set_current(ecs);

        let mut to_disconnect = Vec::new();
        for i in 0..self.my_clients.len() {
            if let Some(client) = self.my_clients.get_mut(i) {
                let player_entity = client.entity.as_ref().unwrap();
                let mut delta_res = if let Some(idx) = client.last_state {
                    self.snapshotter.get_delta(idx as usize, player_entity)
                } else {
                    self.snapshotter.get_full_snapshot(player_entity)
                };

                match delta_res {
                    Ok(mut delta) => {
                        let msg = protocol::NetMessageContent::Delta(DeltaSnapshotInfo {
                            delta,
                            old_state: client.last_state,
                            // Don't worry it is ok for now :D
                            new_state: self.snapshotter.get_current_index() as u8,
                        });
                        self.send_to_client(i, msg);
                    }
                    Err(SnapshotError::ClientCaughtUp) => {
                        info!("To disconnect!");
                        to_disconnect.push(i);
                    }
                    Err(e) => error!("{}", e),
                }
            }
        }

        for i in to_disconnect {
            info!("Will disconnect player {}", i);
            if !self.my_clients.remove(i) {
                error!("Could not remove player {}", i);
            }
        }
    }

    fn handle_client_message(client: &Client, packet: Packet) -> Option<Event> {
        match packet.content {
            protocol::NetMessageContent::MoveCommand(direction) => {
                Some(Event::GameEvent(GameEvent::Move(direction)))
            }
            protocol::NetMessageContent::LookAtCommand(direction) => {
                Some(Event::GameEvent(GameEvent::LookAt(direction)))
            }
            _ => None,
        }
    }

    /// This is called when a ConnectionRequest message is received
    /// It will reply with either connection accepted or connection refused
    /// and add the client to our map of clients.
    ///
    /// If a client is already in the map, it should reply connection
    /// accepted. The reason is that the connection acception message
    /// might have been lost so the client thinks it is still trying to connect
    fn handle_connection_request(&mut self, addr: SocketAddr, ecs: &mut ECS) {
        info!("Handle new connection request from {}", addr);

        let (to_send, client_id) = {
            if let Some(id) = self.get_client_id(addr) {
                info!("Client was already connected, resend ConnectionAccepted");
                (protocol::NetMessageContent::ConnectionAccepted, Some(id))
            } else {
                // in that case we need to find an empty slot. If available,
                // return connection accepted.

                match self.my_clients.add(Client {
                    addr,
                    last_rec_seq_number: 0,
                    last_sent_seq_number: 0,
                    last_state: None,
                    entity: None,
                }) {
                    Some(i) => {
                        info!("New player connected: Player {}!", i);

                        // Now we have a new client, let's create a new player entity
                        // from the player template.
                        let entity = ecs.new_entity();
                        ecs.components.transforms.set(
                            &entity,
                            TransformComponent {
                                position: Vector3::new(0.0, 0.0, 0.0),
                                rotation: Vector3::new(0.0, 0.0, 0.0),
                                scale: Vector3::new(1.0, 1.0, 1.0),
                            },
                        );
                        ecs.components.models.set(
                            &entity,
                            ModelComponent {
                                mesh_name: "cube".to_string(),
                                texture_name: "white".to_string(),
                            },
                        );

                        self.my_clients.get_mut(i).unwrap().entity = Some(entity);
                        (protocol::NetMessageContent::ConnectionAccepted, Some(i))
                    }

                    None => {
                        info!("Too many clients connected, send ConnectionRefused");
                        (protocol::NetMessageContent::ConnectionRefused, None)
                    }
                }
            }
        };

        if let Some(id) = client_id {
            debug!("Send connection accepted");
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
        let client = self
            .my_clients
            .get_mut(client_id)
            .expect("Something wrong happend here");
        let to_send = protocol::NetMessage {
            target: client.addr,
            content: Packet {
                content: msg,
                seq_number: client.last_sent_seq_number,
                last_known_state: None, // doesn't matter on server->client
            },
        };

        if let Err(e) = self.to_clients.send(to_send) {
            error!("Error in send_to_client = {:?}", e);
        } else {
            client.last_sent_seq_number += 1;
        }
    }

    fn has_client(&self, addr: SocketAddr) -> bool {
        self.my_clients
            .iter()
            .any(|client| client.is_some() && client.as_ref().unwrap().addr == addr)
    }

    fn get_client_id(&self, addr: SocketAddr) -> Option<usize> {
        self.my_clients
            .iter()
            .enumerate()
            .find(|(_, client)| client.is_some() && client.as_ref().unwrap().addr == addr)
            .map(|t| t.0)
    }
}
