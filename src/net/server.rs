use std::net::SocketAddr;
use log::{warn, trace, error, info, debug};
use std::io;
use tokio::prelude::*;
use tokio::net::{UdpFramed, UdpSocket};
use tokio_codec::BytesCodec;
use futures::sync::mpsc as futmpsc;
use tokio::prelude::stream::{SplitSink, SplitStream};
use bytes::{Bytes, BytesMut};

use super::NetworkError;
use futures::try_ready;


use std::sync::mpsc as stdmpsc;

use crate::sync::SharedDeque;
use super::protocol;
use std::thread;

pub struct Server {
    clients: Vec<SocketAddr>,
    max_clients: usize,
    stream: SplitStream<UdpFramed<BytesCodec>>,
    sink: SplitSink<UdpFramed<BytesCodec>>,

    // Communication with rest of gameengine.
    to_clients: SharedDeque<Bytes>,
    to_server: SharedDeque<Bytes>,
}

pub fn start_serving() {

}

/// Will create the futures that will run in tokio runtime.
fn connect(port: usize,
           game_to_net: Box<Stream<Item = (Bytes, SocketAddr), Error = io::Error> + Send>,
           mut net_to_game: SharedDeque<(BytesMut, SocketAddr)>)
    -> Result<Box<Stream<Item = (), Error = io::Error> + Send>, Box<std::error::Error>> {

        let addr = format!("127.0.0.1:{}", port).parse()?;
        let socket = UdpSocket::bind(&addr)?;

        let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

        // All bytes from `game_to_net` will go to the `addr` specified in our
        // argument list. Like with TCP this is spawned concurrently
        let forward = game_to_net
            .forward(sink)
            .then(|result| {
                if let Err(e) = result {
                    println!("failed to write to socket: {}", e)
                }
                Ok(())
            });

        let receive = stream.map(move |received| {
            net_to_game.push(received);
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

impl Server {

    pub fn connect(port: usize,
                   max_clients: usize) -> 
        Result<(Server, SharedDeque<Bytes>, SharedDeque<Bytes>), Box<std::error::Error>> {


            let mut clients = Vec:: new();
            clients.reserve_exact(max_clients);

            let addr = format!("127.0.0.1:{}", port).parse()?;
            let socket = UdpSocket::bind(&addr)?;

            let (sink, stream) = UdpFramed::new(socket, BytesCodec::new())
                .split();

            let to_server = SharedDeque::new(1024);
            let to_clients = SharedDeque::new(1024);

            let server = Server {
                clients,
                max_clients,
                sink,
                stream,
                to_server: to_server.clone(),
                to_clients: to_clients.clone(),
            };

            Ok((server, to_clients, to_server))
        }


    fn handle_new_client(&mut self, addr: SocketAddr, msg: BytesMut) {
        println!("New client({:?} with message {:?}", addr, msg);

        let incoming_msg = protocol::deserialize(msg.into());
        if let Err(e) = incoming_msg {
            error!("Incoming message from {} is erroneous = {:?}", addr, e);
            return;
        }
        let incoming_msg = incoming_msg.unwrap();
        if let protocol::NetMessage::ConnectionRequest = incoming_msg {

            if self.clients.len() < self.max_clients {
                // Welcome my man.
                let packed = protocol::serialize(protocol::NetMessage::ConnectionAccepted)
                    .expect("Cannot serialize ConnectionAccepted message");
                self.sink.start_send((packed, addr));
                info!("Connection accepted for {:?}", addr); 
                self.clients.push(addr);
            } else {
                let packed = protocol::serialize(protocol::NetMessage::ConnectionRefused)
                    .expect("Cannot serialize ConnectionRefused message");
                self.sink.start_send((packed, addr));
                info!("Connection refused for {:?}", addr); 
            }
        } else {
            trace!("Client {} is not connected but sent {:?}", addr, incoming_msg);
        }
    }
}


// Server will be ran in a tokio loop so it must implement Future.
impl Future for Server {

    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        // Will check for update from clients (UDP socket).
        while let Async::Ready(data) = self.stream.poll()? {

            match data {
                Some((message, addr)) => {

                    // Here we need to check where the addr is known. If yes, then
                    // send the event to our game. If no, we need to see if it is a
                    // connection request.
                    // replace by some find
                    let mut found = false;
                    for client in &self.clients {
                        if *client == addr {
                            trace!("Got a message from {:?}", client);
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        self.handle_new_client(addr, message);
                    } else {
                        // TODO maybe handle all the messages at the same time so that
                        // mutex is only locked once.
                        self.to_server.push(message.into());
                    }
                },
                None => {
                    warn!("Received None in Server::poll");
                    return Ok(Async::Ready(()));
                },
            }
        }

        println!(" I A M H E R E");

        // process state from server-side.
        // TODO that can't be good
        let state = self.to_clients.drain();
        debug!("will send {} messages", state.len());
        for el in state {
            for client in self.clients.iter() {
                self.sink.start_send((el.clone(), *client));
            }
        }

        // Send available data to clients.
        try_ready!(self.sink.poll_complete());
        Ok(Async::NotReady)
    }
}

fn read_channel(mut tx: futmpsc::Sender<Bytes>, mut rx: stdmpsc::Receiver<Bytes>) {

    loop {
        let d = rx.recv().unwrap();
        tx = match tx.send(d).wait() {
            Ok(tx) => tx,
            Err(e) => {
                error!("Error in read_channel = {:?}", e);
                break;
            }
        }
    }

}
