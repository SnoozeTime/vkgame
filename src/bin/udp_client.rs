use tokio::net::{UdpFramed, UdpSocket};
use std::net::SocketAddr;
use tokio::prelude::*;
use tokio_codec::BytesCodec;
use std::io;
use futures::sync::mpsc;

fn main() -> Result<(), Box<std::error::Error>> {
    println!("hello, world!");

    let src_addr = "127.0.0.1:8081".parse()?;
    let target_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let socket = UdpSocket::bind(&src_addr)?;

    let (stdin_tx, stdin_rx) = mpsc::channel(0);

    let (sink, stream) = UdpFramed::new(socket, BytesCodec::new()).split();

    let forward_sink = stdin_rx
        .map(move |chunk| (chunk, target_addr))
        .sink_map_err(|_| io::Error::new(io::ErrorKind::NotFound, "noooo"))
        .forward(sink)
        .then(|result| {
            if let Err(e) = result {
                println!("Cannot send data to socket = {:?}", e);
            }
        });

    let receive = stream.filter_map(move |(chunk, src)| {
        if src == target_addr {
            Some(chunk)
        } else {
            None
        }
    });

    let stream = future::lazy(|| {
        tokio::spawn(forward_sink);
        future::ok(receive)
    }).flatten_stream();

   tokio::run(stream.for_each(move |chunk | println!("{:?}", chunk))
              .map_err(|e| println!("Err = {:?}", e)));
    Ok(())
}
