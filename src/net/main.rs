use serde_derive::{Deserialize, Serialize};
use serde::{Deserialize, Serialize};
use rmp_serde::{Deserializer, Serializer};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Human {
    name: String,
    age: u32,
}

use tokio::io;
use tokio::codec::Framed;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use futures::sync::mpsc;
use futures::future::{self, Either};
use futures::try_ready;
use bytes::{BytesMut, Bytes, BufMut};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

type Tx = mpsc::UnboundedSender<Bytes>;
type Rx = mpsc::UnboundedReceiver<Bytes>;

mod codec;
// Data shared between all peers in the server.
struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

// Struct for each connected clients.
struct Peer {
    socket: Framed<TcpStream, codec::NetstringCodec>,
    //// First line when connecting to the server.
    //name: BytesMut,

    //// TCP socket wrapped in the Line codecs. Handles
    //// sending and receiving lines on the socket.
    //lines: Lines,

    //// Shared state
    //state: Arc<Mutex<Shared>>,

    //// Receive half of the message channel.
    //rx: Rx,

    //// client socket address
    //addr: SocketAddr,
}


/// Line based code
#[derive(Debug)]
struct Lines {
    socket: TcpStream,
    // buffer used when reading from the socket.
    rd: BytesMut,

    // buffer used to stage data before writing it to the socket
    wr: BytesMut,
}



impl Shared {

    /// Create a new, empty, instance of `Shared`.
    fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }
}

impl Peer {

    fn new(socket: Framed<TcpStream, codec::NetstringCodec>) -> Peer {

        Peer {
            socket,
        }
    }
}

impl Future for Peer {

    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        while let Async::Ready(line) = self.socket.poll()? {

            println!("Received line: {:?}", line);
        }

        // As always, it is important to not just return `NotReady` without
        // ensuring an inner future also returned `NotReady`.
        //
        // We know we got a `NotReady` from either `self.rx` or `self.lines`, so
        // the contract is respected.
        Ok(Async::NotReady)
    }
}

// Spawn a task to manage the socket.
fn process(socket: TcpStream, state: Arc<Mutex<Shared>>) {
    // transform our stream of bytes to stream of frames.
    let framed_sock = Framed::new(socket, codec::NetstringCodec::new(123, true));

    let connection = Peer::new(framed_sock).map_err(|e| {

            println!("connection error = {:?}", e);

        });
    // spawn the task. Internally, this submits the task to a thread pool
    tokio::spawn(connection);
}

fn main() -> Result<(), Box<std::error::Error>> {

    let mut buf = Vec::new();
    let val = Human { age: 42, name: "John".into() };

    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
    dbg!(buf);

    let buf2 = vec![ 146,
    164,
    74,
    111,
    104,
    110,
    42,
    12,
    5,
    123,
    ];

    let h = rmp_serde::from_slice::<Human>(&buf2);
    dbg!(h);

    let state = Arc::new(Mutex::new(Shared::new()));
    let addr = "127.0.0.1:6142".parse()?;
    let listener = TcpListener::bind(&addr)?;
    let server = listener.incoming().for_each(move |socket| {
        process(socket, state.clone());
        Ok(())
    })
    .map_err(|err| {
        println!("accept error = {:?}", err);
    });

    println!("Running on localhost:6142");
    tokio::run(server);

    Ok(())
}



