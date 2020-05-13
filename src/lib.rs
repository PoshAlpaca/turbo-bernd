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

use http::{Response, Status};
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
) -> Result<http::Response, middleware::Error> {
    for current in middleware {
        match current.answer(&request) {
            Err(middleware::Error::NotFound) => continue,
            res => return res,
        };
    }

    Err(middleware::Error::NotFound)
}

fn create_response(s: String, middleware: &[Box<dyn Middleware>]) -> Response {
    match http::Request::parse(&s) {
        Ok(req) => match get_response(middleware, &req) {
            Ok(res) => res,
            Err(e) => match e {
                middleware::Error::MethodNotAllowed => Response::new(Status::MethodNotAllowed),
                middleware::Error::NotFound => Response::new(Status::NotFound),
            },
        },
        Err(e) => match e {
            http::Error::UnsupportedVersion => Response::new(Status::VersionNotSupported),
            http::Error::UnknownMethod => Response::new(Status::BadRequest),
            http::Error::MalformedRequest => Response::new(Status::BadRequest),
        },
    }
}

fn handle_client(mut stream: TcpStream, middleware: &[Box<dyn Middleware>]) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer[..]).unwrap();

    let req_str = str::from_utf8(&buffer).unwrap();
    let request = String::from(req_str);

    // TODO
    let request_copy = request.clone();
    let first_line = request_copy.splitn(2, "\r\n").next().unwrap();
    let response = create_response(request, &middleware);

    let s = format!("{} => {}", first_line, response.status);
    info!("{}", s);

    let _ = stream.write(format!("{}", response).as_bytes());
}
