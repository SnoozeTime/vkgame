use std::net::SocketAddr;
use std::io;
use tokio::prelude::*;
use tokio::net::{UdpFramed, UdpSocket};
use tokio_codec::BytesCodec;
use tokio::prelude::stream::{SplitSink, SplitStream};
use bytes::BytesMut;

const CLIENT_NB: usize = 8;

use super::NetworkError;

pub struct Server {
    clients: Vec<SocketAddr>,
    sink: SplitSink<UdpFramed<BytesCodec>>,
    stream: SplitStream<UdpFramed<BytesCodec>>,
}

impl Server {
    
    pub fn connect(port: usize) -> Result<Self, Box<std::error::Error>> {
        let mut clients = Vec:: new();
        clients.reserve_exact(CLIENT_NB);

        let addr = format!("127.0.0.1:{}", port).parse()?;
        let socket = UdpSocket::bind(&addr)?;

        let (sink, stream) = UdpFramed::new(socket, BytesCodec::new())
            .split();

        Ok(Server {
            clients,
            sink,
            stream,
        })
    }


    fn handle_new_client(&mut self, addr: SocketAddr, msg: BytesMut) {
        println!("New client({:?} with message {:?}", addr, msg)
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
        
        Ok(Async::NotReady)
    }
}
