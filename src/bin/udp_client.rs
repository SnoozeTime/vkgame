use tokio::net::{UdpFramed, UdpSocket};
use std::net::SocketAddr;
use tokio::prelude::*;
use std::io;
use futures::sync::mpsc;
use bytes::Bytes;
use std::thread;

fn main() -> Result<(), Box<std::error::Error>> {
    println!("hello, world!");

    let target_addr: SocketAddr = "127.0.0.1:8080".parse()?;

    let (stdin_tx, stdin_rx) = mpsc::channel(0);
    thread::spawn(|| read_stdin(stdin_tx));
    let stdin_rx = stdin_rx.map_err(|_| panic!("errors not possible on rx"));

    let stream = udp::connect(&target_addr, Box::new(stdin_rx))?;

    let mut stdout = io::stdout();
    tokio::run(stream
        .for_each(move |chunk| stdout.write_all(&chunk))
        .map_err(|e| println!("Err = {:?}", e)));

    //   tokio::run(stream.for_each(move |chunk | println!("{:?}", chunk))
    //              .map_err(|e| println!("Err = {:?}", e)));
    Ok(())
}
mod udp {
    use std::error::Error;
    use std::io;
    use std::net::SocketAddr;

    use tokio_codec::BytesCodec;
    use bytes::{Bytes, BytesMut};
    use tokio;
    use tokio::net::{UdpFramed, UdpSocket};
    use tokio::prelude::*;


    pub fn connect(
        &addr: &SocketAddr,
        stdin: Box<Stream<Item = Bytes, Error = io::Error> + Send>,
        ) -> Result<Box<Stream<Item = BytesMut, Error = io::Error> + Send>, Box<Error>> {
        // We'll bind our UDP socket to a local IP/port, but for now we
        // basically let the OS pick both of those.
        let addr_to_bind = if addr.ip().is_ipv4() {
            "0.0.0.0:0".parse()?
        } else {
            "[::]:0".parse()?
        };
        let udp = match UdpSocket::bind(&addr_to_bind) {
            Ok(udp) => udp,
            Err(_) => Err("failed to bind socket")?,
        };

        // Like above with TCP we use an instance of `Bytes` codec to transform
        // this UDP socket into a framed sink/stream which operates over
        // discrete values. In this case we're working with *pairs* of socket
        // addresses and byte buffers.
        let (sink, stream) = UdpFramed::new(udp, BytesCodec::new()).split();

        // All bytes from `stdin` will go to the `addr` specified in our
        // argument list. Like with TCP this is spawned concurrently
        let forward_stdin = stdin
            .map(move |chunk| (chunk, addr))
            .forward(sink)
            .then(|result| {
                if let Err(e) = result {
                    println!("failed to write to socket: {}", e)
                }
                Ok(())
            });

        // With UDP we could receive data from any source, so filter out
        // anything coming from a different address
        let receive = stream.filter_map(move |(chunk, src)| {
            if src == addr {
                Some(chunk.into())
            } else {
                None
            }
        });

        let stream = Box::new(
            future::lazy(|| {
                tokio::spawn(forward_stdin);
                future::ok(receive)
            })
            .flatten_stream(),
            );
        Ok(stream)
    }
}

// Our helper method which will read data from stdin and send it along the
// sender provided.
fn read_stdin(mut tx: mpsc::Sender<Bytes>) {
    let mut stdin = io::stdin();
    loop {
        let mut buf = vec![0; 1024];
        let n = match stdin.read(&mut buf) {
            Err(_) | Ok(0) => break,
            Ok(n) => n,
        };
        buf.truncate(n);
        tx = match tx.send(buf.into()).wait() {
            Ok(tx) => tx,
            Err(_) => break,
        };
    }
}
