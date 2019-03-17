use clap::{App, Arg};
use log::info;

fn main() {
    env_logger::init();

    // Extract the server address from the command-line.
    let matches = App::new("Client")
        .version("0.1")
        .author("Benoit Eudier")
        .arg(Arg::with_name("connect")
             .short("c")
             .long("connect")
             .required(false)
             .takes_value(true)
             .help("IP address of the server"))
        .get_matches();

    let addr = matches.value_of("connect").unwrap_or("localhost:8080");

    info!("Will connect to server at {}", addr);

}
