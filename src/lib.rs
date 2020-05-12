use crossbeam::scope;
use log::{error, info};
use std::{
    env,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    str,
};

pub mod http;
pub mod middleware;
pub mod routing;

use middleware::Middleware;

struct Config {
    port: String,
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &str> {
        if args.len() < 2 {
            return Err("too few arguments, please include port");
        }

        let port = args[1].clone();

        Ok(Config { port })
    }
}

pub struct Application {
    config: Config,
    middleware: Vec<Box<dyn Middleware>>,
}

impl Application {
    pub fn new(middleware: Vec<Box<dyn Middleware>>) -> Application {
        let args: Vec<String> = env::args().collect();
        let config = Config::new(&args).unwrap();

        Application { config, middleware }
    }

    pub fn run(&self) {
        info!(
            "Starting {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let address = format!("127.0.0.1:{}", self.config.port);

        info!("Listening at: {}", address);
        let listener = TcpListener::bind(address).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            scope(|s| {
                s.spawn(move |_| {
                    handle_client(stream, &self.middleware);
                });
            })
            .unwrap();
        }
    }
}

fn get_response(
    middleware: &[Box<dyn Middleware>],
    request: &http::Request,
) -> Result<http::Response, http::Status> {
    for current in middleware {
        if let Ok(res) = current.answer(&request) {
            return Ok(res);
        } else {
            continue;
        }
    }

    Err(http::Status::NotFound)
}

fn create_response(
    s: String,
    middleware: &[Box<dyn Middleware>],
) -> Result<http::Response, http::Status> {
    let request = http::Request::parse(&s).or(Err(http::Status::BadRequest))?;

    get_response(middleware, &request)
}

fn handle_client(mut stream: TcpStream, middleware: &[Box<dyn Middleware>]) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer[..]).unwrap();

    let req_str = str::from_utf8(&buffer).unwrap();

    let request = String::from(req_str);

    // TODO
    let request_copy = request.clone();
    let first_line = request_copy.splitn(2, "\r\n").next().unwrap();

    let response = match create_response(request, &middleware) {
        Ok(r) => {
            info!("{} => {}", first_line, r.status);
            r
        }
        // TODO: More custom error types, which diff. cases need to be handled?
        Err(e) => {
            error!("{} => {}", first_line, e);

            http::Response {
                version: http::Version::OneDotOne,
                status: e,
                headers: http::Headers {
                    headers: Vec::new(),
                },
                body: "".to_string(),
            }
        }
    };

    let _ = stream.write(format!("{}", response).as_bytes());
}
