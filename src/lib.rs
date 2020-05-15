#![feature(test)]

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

use http::{Response, ResponseClass, Status};
use middleware::Middleware;

pub struct Config {
    port: u16,
}

impl Config {
    pub fn new(port: u16) -> Config {
        Config { port }
    }

    pub fn from_args() -> Config {
        let args: Vec<String> = env::args().collect();
        let port = u16::from_str_radix(&args[1], 10).expect("Supplied port is invalid");
        Config { port }
    }
}

pub struct Application {
    middleware: Vec<Box<dyn Middleware>>,
}

impl Application {
    pub fn new(middleware: Vec<Box<dyn Middleware>>) -> Application {
        Application { middleware }
    }

    pub fn run(&self, config: Config) {
        info!(
            "Starting {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let address = format!("127.0.0.1:{}", config.port);

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

    match response.class() {
        ResponseClass::Informational => info!("{}", s),
        ResponseClass::Successful => info!("{}", s),
        ResponseClass::Redirection => info!("{}", s),
        ResponseClass::ClientError => error!("{}", s),
        ResponseClass::ServerError => error!("{}", s),
    }

    let _ = stream.write(format!("{}", response).as_bytes());
}
