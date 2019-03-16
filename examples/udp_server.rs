use tokio::net::{UdpFramed, UdpSocket};
use std::net::SocketAddr;
use tokio::prelude::*;
use tokio_codec::BytesCodec;
use std::io;
use tokio::prelude::stream::{SplitSink, SplitStream};

use std::collections::HashMap;
use future::Future;


/*

    connection:
    the client will send a connection request to the server with an username.
    if server receives, it will:
    - check the ip address and names. if unique, it will accept the connection request and create
      a new client in pending state. then it will send connection accepted message.
    - if not ok, will sned connection refused only once.

    if connection if accepted, the .... to be continued!

*/


struct Server {
    sink: SplitSink<UdpFramed<BytesCodec>>,
    stream: SplitStream<UdpFramed<BytesCodec>>,
    buf: Vec<u8>,
    to_send: Option<(usize, SocketAddr)>,

}

enum clientstate {
    disconnected,
    pending,
    connected,
}

struct client {
    incoming_seq_nb: usize,
    outgoing_seq_nb: usize,
    name: String,
}

impl Server {
    pub fn new(socket: UdpSocket) -> Self {
        let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

        Server {
            sink,
            stream,
            buf: vec![],
            to_send: None,
        }
    }
}

impl Future for Server {

    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        while let Async::Ready(data) = self.stream.poll()? {
            match data {
                Some((msg, addr)) => {
                    println!("received {:?} from {:?}", msg, addr)

                    //if !clients.contains_key(&addr) {
                    //    // first message. needs to be a connection request with a name.
                    //    // otherwise just drop.

                    //    // then sends ok to it until it is connected.
                    //}

                },
                None => return Ok(Async::Ready(())),
            }
        }

        Ok(Async::NotReady)
    }
}

fn main() -> Result<(), Box<std::error::Error>> {
    println!("hello, world!");

    let addr = "127.0.0.1:8080".parse()?;
    let socket = UdpSocket::bind(&addr)?;

    let server = Server::new(socket);
    tokio::run(server.map_err(|e| println!("server error = {:?}", e)));

    Ok(())
}
