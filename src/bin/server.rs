use clap::{App, Arg};
use log::{error, info, trace};
use std::time::{Duration, Instant};
use twgraph::scene::{Scene, NetworkScene};

/// Validator for clap
fn is_usize(v: String) -> Result<(), String> {
    if let Err(_) = v.parse::<usize>() {
        return Err("The value should represent an usize".to_string())
    }

    Ok(())
}


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
             .default_value("8080")
             .validator(is_usize)
             .help("Port of the server"))
        .arg(Arg::with_name("number")
                .short("n")
                .long("number")
                .required(false)
                .takes_value(true)
                .default_value("8")
                .validator(is_usize)
                .help("Number of players"))
        .get_matches();

    // clap has already done the validation and default value.
    let port = matches.value_of("port").unwrap().parse().unwrap();
    let nb = matches.value_of("number").unwrap().parse().unwrap();

    info!("Will connect on port {}, with {} players", port, nb);

    let fixed_time_stamp = Duration::new(0, 16666667);
    let mut previous_clock = Instant::now();
    let mut accumulator = Duration::new(0, 0);

    // The scene will contains all the systems, including the network stack.
    // Here, no need for Scene stack or anything fancy.
    let mut scene = NetworkScene::new(port, nb);
    
    'game_loop: loop {

        while accumulator > fixed_time_stamp {
            accumulator -= fixed_time_stamp;

            // Do the work.
            trace!("Run server frame");
            scene.update(fixed_time_stamp);
        }

        accumulator += Instant::now() - previous_clock;
        previous_clock = Instant::now();
    }

    // Ok(())
}
