use twgraph::net::protocol::*;
use std::error::Error;
use log::{debug, info, trace};
use std::thread;
use std::time::Duration;
use twgraph::net::start_connecting;


const NB_TRY: u32 = 10;

fn main() -> Result<(), Box<Error>> { 

    env_logger::init();
    let (mut from_server, to_server) = start_connecting("127.0.0.1:8080".parse()?)?;

    // Connection to server. Try to send message every seconds until it receives
    // a connection accepted or a connection refused.
    info!("Will connect to the game server");
    let is_connected: bool = {

        let mut try_nb = 0u32;
        let mut res = false;
        'connection: loop {
            if try_nb >= NB_TRY {
                info!("Timed out during connection to server");
                break 'connection;
            }

            to_server.send(NetMessageContent::ConnectionRequest);

            thread::sleep(Duration::from_secs(1));
            let evs = from_server.drain();
            // ok we might lose some events here. It's alright, the server
            // is sending state every loop and if message needs to be reliably sent,
            // the server will resend it.
            for ev in evs {
                match ev {
                    NetMessageContent::ConnectionAccepted => {
                        res = true;
                        break 'connection;
                    },
                    NetMessageContent::ConnectionRefused => {
                        info!("Received connection refused");
                        break 'connection;
                    },
                    _ => debug!("Received {:?} when connecting. That is strange", ev),
                }
            }

            try_nb += 1;
        }

        res
    };

    if !is_connected {
        return Ok(());
    }

    info!("Connected to server");
    loop {

        let evs = from_server.drain();
        for ev in evs {
            trace!("{:?}", ev);
        }

        thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}

