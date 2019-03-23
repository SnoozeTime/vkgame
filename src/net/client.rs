use bytes::{Bytes, BytesMut};
use futures::sync::mpsc as futmpsc;
use log::{debug, error, info, trace, warn};
use std::io;
use std::net::SocketAddr;
use std::thread;
use tokio::net::{UdpFramed, UdpSocket};
use tokio::prelude::stream::{SplitSink, SplitStream};
use tokio::prelude::*;
use tokio_codec::BytesCodec;

use super::protocol;
use super::protocol::Packet;

use std::sync::mpsc as stdmpsc;
use std::time::Duration;

use super::NetworkError;
use crate::sync::SharedDeque;

const NB_TRY: u32 = 10;

/// Connect to the remote server and returns interfaces to send and receive
/// messages
///
/// As opposed to the server, we don't need to attach the SocketAddr that
/// sent the message and we don't need to specify the SocketAddr when sending
/// because this is known at start time.
pub fn start_connecting(
    server_addr: SocketAddr,
) -> Result<(SharedDeque<Packet>, stdmpsc::Sender<Packet>), Box<std::error::Error>> {
    info!("Start connecting to {}", server_addr);
    // interfaces
    let net_to_game = SharedDeque::new(1024);
    let mut net_to_game_clone = net_to_game.clone();
    let (int_tx, int_rx) = futmpsc::channel(1024);
    let (tx, rx) = stdmpsc::channel();
    let int_rx = int_rx.map_err(|_| panic!("Error not possible on rx"));

    thread::spawn(move || read_channel(int_tx, rx));

    let async_stuff = connect(server_addr, Box::new(int_rx))?;
    thread::spawn(move || {
        tokio::run(
            async_stuff
                .for_each(move |buf| {
                    match protocol::deserialize(buf.into()) {
                        Ok(unpacked) => {
                            net_to_game_clone.push(unpacked);
                        }
                        Err(e) => {
                            error!(
                                "Received malformed message from {}, error = {:?}",
                                server_addr, e
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
    server_addr: SocketAddr,
    game_to_net: Box<Stream<Item = Bytes, Error = io::Error> + Send>,
) -> Result<Box<Stream<Item = BytesMut, Error = io::Error> + Send>, Box<std::error::Error>> {
    let addr = "127.0.0.1:0".parse()?;
    let socket = UdpSocket::bind(&addr)?;

    let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

    // All bytes from `game_to_net` will go to the `addr` specified in our
    // argument list. Like with TCP this is spawned concurrently
    let forward = game_to_net
        .map(move |chunk| (chunk, server_addr))
        .forward(sink)
        .then(|result| {
            if let Err(e) = result {
                error!("failed to write to socket: {}", e)
            }
            Ok(())
        });

    // TODO Also filter packet out of order
    let receive = stream.filter_map(move |(chunk, src)| {
        if src == server_addr {
            Some(chunk)
        } else {
            None
        }
    });

    let stream = Box::new(
        future::lazy(|| {
            tokio::spawn(forward);
            future::ok(receive)
        })
        .flatten_stream(),
    );

    Ok(stream)
}

fn read_channel(mut tx: futmpsc::Sender<Bytes>, rx: stdmpsc::Receiver<Packet>) {
    loop {
        match rx.recv() {
            Ok(d) => {
                // if cannot serialize here, we have a problem...

                let packed = protocol::serialize(d)
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
            Err(e) => {
                error!("Error on read channel = {:?}", e);
                break;
            }
        }
    }
}

/// The actual game system that will be running in the main loop
pub struct ClientSystem {
    /// Messages incoming from the server.
    from_server: SharedDeque<Packet>,

    /// Queue to send to server
    to_server: stdmpsc::Sender<Packet>,

    last_rec_seq_number: u32,
    last_known_state: Option<u8>,
}

impl ClientSystem {
    pub fn connect(addr: SocketAddr) -> Result<Self, NetworkError> {
        let (mut from_server, to_server) = start_connecting(addr).map_err(|e| {
            // meh, need to find a better way to extract tokio errors
            error!("{:?}", e);
            NetworkError::CannotConnectToServer
        })?;

        // Connection to server. Try to send message every seconds until it receives
        // a connection accepted or a connection refused.
        info!("Will connect to the game server");
        let is_connected: bool = {
            let mut try_nb = 0u32;
            let mut res = false;
            'connection: loop {
                if try_nb >= NB_TRY {
                    info!("Timed out during connection to server");
                    break 'connection;
                }

                if let Err(e) = to_server.send(Packet {
                    content: protocol::NetMessageContent::ConnectionRequest,
                    seq_number: 0,
                    last_known_state: None,
                }) {
                    error!("{:?}", e);
                }

                thread::sleep(Duration::from_secs(1));
                let evs = from_server.drain();
                // ok we might lose some events here. It's alright, the server
                // is sending state every loop and if message needs to be reliably sent,
                // the server will resend it.
                for ev in evs {
                    match ev.content {
                        protocol::NetMessageContent::ConnectionAccepted => {
                            res = true;
                            break 'connection;
                        }
                        protocol::NetMessageContent::ConnectionRefused => {
                            info!("Received connection refused");
                            break 'connection;
                        }
                        _ => debug!("Received {:?} when connecting. That is strange", ev),
                    }
                }

                try_nb += 1;
            }

            res
        };

        if is_connected {
            Ok(Self {
                to_server,
                from_server,
                last_rec_seq_number: 0,
                last_known_state: None,
            })
        } else {
            Err(NetworkError::CannotConnectToServer)
        }
    }

    /// Will get the latest events that were sent from the server
    pub fn poll_events(&mut self) {
        let events = self.from_server.drain();

        for ev in events {
            if self.last_rec_seq_number >= ev.seq_number {
                error!(
                    "Received packet out of order: last_rec_seq_number {} > packet.seq_number {}",
                    self.last_rec_seq_number, ev.seq_number
                );
            } else {
                self.last_rec_seq_number = ev.seq_number;
                debug!("Client system received {:?}", ev);
            }
        }
    }
}
