use clap::{App, Arg};
use log::{error, info, trace};
use winit::EventsLoop;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<std::error::Error>> {
    env_logger::init();

    // Extract the server address from the command-line.
    let matches = App::new("Serer")
        .version("0.1")
        .author("Benoit Eudier")
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .required(false)
             .takes_value(true)
             .help("Port of the server"))
        .get_matches();

    let port_str = matches.value_of("port").unwrap_or("8080");

    let port: Result<usize,_> = port_str.parse();
    if let Err(err) = port {
        error!("Cannot parse the port as number: {}", port_str);
        return Err(Box::new(err));
    }

    let port = port.unwrap();
    info!("Will connect on port {}", port);

    // The server will run a game loop without the renderer.  
    info!("Initialize systems");

    info!("System initialized");

    
    let mut fixed_time_stamp = Duration::new(0, 16666667);
    let mut previous_clock = Instant::now();
    let mut accumulator = Duration::new(0, 0);

    'game_loop: loop {

        while accumulator > fixed_time_stamp {
            accumulator -= fixed_time_stamp;

            // Do the work.
            trace!("Run server frame");
        }

        accumulator += Instant::now() - previous_clock;
        previous_clock = Instant::now();
    }

    Ok(())
}
