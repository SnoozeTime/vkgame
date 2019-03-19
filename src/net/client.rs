use std::net::SocketAddr;
use log::{warn, trace, error, info};
use std::io;
use tokio::prelude::*;
use tokio::net::{UdpFramed, UdpSocket};
use tokio_codec::BytesCodec;
use futures::sync::mpsc;
use tokio::prelude::stream::{SplitSink, SplitStream};
use bytes::{Bytes, BytesMut};
use std::error::Error;

use super::NetworkError;
use futures::try_ready;

use crate::sync::SharedDeque;
use super::protocol;
use super::protocol::NetMessage;

pub struct Client {

    server_addr: SocketAddr,
    stream: SplitStream<UdpFramed<BytesCodec>>,
    sink: SplitSink<UdpFramed<BytesCodec>>,

    to_remote: SharedDeque<NetMessage>,
    to_game: SharedDeque<NetMessage>,
}

impl Client {

    pub fn connect(server_addr: SocketAddr)
        -> Result<(Client, SharedDeque<Bytes>, SharedDeque<Bytes>), Box<Error>> {

            let addr = "127.0.0.1:8081".parse()?;
            let socket = UdpSocket::bind(&addr)?;
            info!("Socket is bound");

            let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

            let to_remote = SharedDeque::new(1024);
            let to_game = SharedDeque::new(1024);

            Ok(Client {
                server_addr,
                sink,
                stream,
                to_remote,
                to_game,
            })
        }
}

impl Future for Client {

    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {

        // Receive all messages from server and send them to game.
        while let Async::Ready(data) = self.stream.poll()? {

            match data {
                Some((message, addr)) if addr == self.server_addr => {
                    match protocol::deserialize(message) {
                        Ok(msg) => to_game.push(msg),
                        Err(e) => error!("Error when deserializing client message = {:?}", e),
                        }
                    }
                },
                Some((_, addr) => {
                    // Not the server is sending. Wtf.
                    warn!("Received message from unknown host = {:?}", addr);
                },
                None => {
                    warn!("Received None is Client::poll");
                    return Ok(Async::Ready());
                }

                }
                }


                }



                }
