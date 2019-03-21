use std::net::SocketAddr;
use log::{warn, trace, error, info, debug};
use std::io;
use tokio::prelude::*;
use tokio::net::{UdpFramed, UdpSocket};
use tokio_codec::BytesCodec;
use futures::sync::mpsc as futmpsc;
use tokio::prelude::stream::{SplitSink, SplitStream};
use bytes::{Bytes, BytesMut};

use std::thread;
use super::protocol;

use std::sync::mpsc as stdmpsc;

use crate::sync::SharedDeque;
use super::NetworkError;

/// Connect to the remote server and returns interfaces to send and receive 
/// messages
///
/// As opposed to the server, we don't need to attach the SocketAddr that
/// sent the message and we don't need to specify the SocketAddr when sending
/// because this is known at start time.
pub fn start_connecting(server_addr: SocketAddr)
    -> Result<(SharedDeque<protocol::NetMessageContent>, stdmpsc::Sender<protocol::NetMessageContent>), Box<std::error::Error>> {
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
                        Ok(unpacked) => net_to_game_clone.push(unpacked),
                        Err(e) => {
                            error!("Received malformed message from {}, error = {:?}",
                                   server_addr,
                                   e);
                        },
                    }

                    Ok(())
                }
                )       
                .map_err(|e| error!("{:?}", e)));
        });

        Ok((net_to_game, tx))
    }

/// Will create the futures that will run in tokio runtime.
fn connect(server_addr: SocketAddr,
           game_to_net: Box<Stream<Item = Bytes, Error = io::Error> + Send>,)
    ->  Result<Box<Stream<Item = BytesMut, Error = io::Error> + Send>,
    Box<std::error::Error>> {

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

fn read_channel(mut tx: futmpsc::Sender<Bytes>,
                rx: stdmpsc::Receiver<protocol::NetMessageContent>) {

    loop {

        match rx.recv() {
            Ok(d) => {
                // if cannot serialize here, we have a problem...

                let packed = protocol::serialize(d).map_err(|e| {
                    error!("Error when unpacking in `read_channel` = {:?}", e);
                    e
                }).unwrap();

                tx = match tx.send(packed).wait() {
                    Ok(tx) => tx,
                    Err(e) => {
                        error!("Error in read_channel = {:?}", e);
                        break;
                    }
                }
            },
            Err(e) => {
                error!("Error on read channel = {:?}", e);
                break;
            }
        }
    }

}
