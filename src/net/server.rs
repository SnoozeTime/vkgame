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

pub fn start_serving(port: usize)
    -> Result<(SharedDeque<protocol::NetMessage>, stdmpsc::Sender<protocol::NetMessage>),
              Box<std::error::Error>> {
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
                        error!("Received malformed message from {}, error = {:?}",
                               client,
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
fn connect(port: usize,
           game_to_net: Box<Stream<Item = (Bytes, SocketAddr), Error = io::Error> + Send>,)
    ->  Result<Box<Stream<Item = (BytesMut, SocketAddr), Error = io::Error> + Send>,
               Box<std::error::Error>> {

        let addr = format!("127.0.0.1:{}", port).parse()?;
        let socket = UdpSocket::bind(&addr)?;

        let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

        // All bytes from `game_to_net` will go to the `addr` specified in our
        // argument list. Like with TCP this is spawned concurrently
        let forward = game_to_net
            .forward(sink)
            .then(|result| {
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

fn read_channel(mut tx: futmpsc::Sender<(Bytes, SocketAddr)>,
                rx: stdmpsc::Receiver<protocol::NetMessage>) {

    loop {
        let d = rx.recv().unwrap();
        
        // if cannot serialize here, we have a problem...
        let packed = d.pack().map_err(|e| {
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
    }

}
