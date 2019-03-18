use std::net::SocketAddr;
use log::error;
use std::io;
use tokio::prelude::*;
use tokio::net::{UdpFramed, UdpSocket};
use tokio_codec::BytesCodec;
use futures::sync::mpsc;
use tokio::prelude::stream::{SplitSink, SplitStream};
use bytes::{Bytes, BytesMut};

use super::NetworkError;
use futures::try_ready;

pub struct Server {
    clients: Vec<SocketAddr>,
    max_clients: usize,
    stream: SplitStream<UdpFramed<BytesCodec>>,
    sink: SplitSink<UdpFramed<BytesCodec>>,

    // Communication with rest of gameengine.
    to_clients: mpsc::UnboundedReceiver<Bytes>,
    to_server: mpsc::UnboundedSender<Bytes>,
}

impl Server {

    pub fn connect(port: usize,
                   max_clients: usize) -> 
        Result<(Server, mpsc::UnboundedSender<Bytes>, mpsc::UnboundedReceiver<Bytes>), Box<std::error::Error>> {


            let mut clients = Vec:: new();
            clients.reserve_exact(max_clients);

            let addr = format!("127.0.0.1:{}", port).parse()?;
            let socket = UdpSocket::bind(&addr)?;

            let (sink, stream) = UdpFramed::new(socket, BytesCodec::new())
                .split();


            // Confusing. 
            // The *_to_clients channel represent a flow of messages from the server
            // to the clients, so the server will send them over UDP
            //
            // The *_to_server will represent a flow of messages from the clients to 
            // the server so the server will receive them over UDP.
            let (tx_to_clients, rx_to_clients) = mpsc::unbounded();
            let (tx_to_server, rx_to_server) = mpsc::unbounded();

            let server = Server {
                clients,
                max_clients,
                sink,
                stream,
                to_server: tx_to_server,
                to_clients: rx_to_clients,
            };

            Ok((server, tx_to_clients, rx_to_server))
        }


    fn handle_new_client(&mut self, addr: SocketAddr, msg: BytesMut) {
        println!("New client({:?} with message {:?}", addr, msg);

        if self.clients.len() < self.max_clients {
            self.sink.start_send((msg.into(), addr));
            self.clients.push(addr);
            println!("Connection accepted") 
        } else {
            println!("Connection refused");
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
                            println!("Got a message from {:?}", client);
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        self.handle_new_client(addr, message);
                    } else {
                        if let Err(err) = self.to_server.unbounded_send(message.into()) {
                            error!("Error sending over unbounded channel = {:?}", err);
                        }
                    }
                },
                None => {
                    println!("Received None in Server::poll");
                    return Ok(Async::Ready(()));
                },
            }
        }

        // process state from server-side.

        // Send available data to clients.
        try_ready!(self.sink.poll_complete());
        Ok(Async::NotReady)
    }
}
