use std::net::SocketAddr;
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
    tx: mpsc::Sender<(Bytes, SocketAddr)>,
    stream: SplitStream<UdpFramed<BytesCodec>>,

    // buffer of messages to send :)
    
}

impl Server {

    pub fn connect(port: usize,
                   max_clients: usize) -> 
        Result<Box<Future<Item = (), Error = io::Error> + Send>, Box<std::error::Error>> {



            let mut clients = Vec:: new();
            clients.reserve_exact(max_clients);

            let addr = format!("127.0.0.1:{}", port).parse()?;
            let socket = UdpSocket::bind(&addr)?;

            let (sink, stream) = UdpFramed::new(socket, BytesCodec::new())
                .split();


            // The sink will be in its own tasks :) And receive the messages
            // to send via the channel.
            let (tx, rx) = mpsc::channel(0); // TODO what is the buffer size doing?
            let rx: Box<Stream<Item = (Bytes, SocketAddr), Error = io::Error> + Send> = Box::new(rx.map_err(|e| panic!("{:?}", e)));

            let forward_msg = rx
                .forward(Box::new(sink))
                .then(|result| {
                    if let Err(e) = result {
                        println!("Failed to write to socket: {:?}", e);
                    }
                    Ok(())
                });


            let server = Server {
                clients,
                max_clients,
                tx,
                stream,
            };

            let fut = Box::new(future::lazy(|| {
                tokio::spawn(forward_msg);
                tokio::spawn(server.map_err(|_| println!("Got an error ..")));
                future::ok(())
            }));

            Ok(fut)
        }


    fn handle_new_client(&mut self, addr: SocketAddr, msg: BytesMut) {
        println!("New client({:?} with message {:?}", addr, msg);

        if self.clients.len() < self.max_clients {
            self.tx.start_send((msg.into(), addr));
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

        // Will check for update from clients.
        while let Async::Ready(data) = self.stream.poll()? {

            match data {
                Some((message, addr)) => {

                    // Here we need to check where the addr is known. If yes, then
                    // send the event to our game. If no, we need to see if it is a
                    // connection request.
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
                    }
                },
                None => {
                    println!("Received None in Server::poll");
                    return Ok(Async::Ready(()));
                },
            }
        }


        // Send available data to clients.
        try_ready!(self.tx.poll_complete().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e.to_string())   
        }));
        Ok(Async::NotReady)
    }
}
