use pretty_env_logger;
use std::env;
use std::process;
use turbo_bernd::Config;

fn main() {
    pretty_env_logger::init_timed();

    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    // we don't return anything in the Ok case so only need to handle Err case
    if let Err(e) = turbo_bernd::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
