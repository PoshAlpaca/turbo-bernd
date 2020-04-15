use chrono::prelude::*;
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
mod thread_pool;

use middleware::{FileMiddleware, Middleware};
use routing::Router;
use thread_pool::ThreadPool;

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
    router: Router,
}

impl Application {
    pub fn new(router: Router) -> Application {
        let args: Vec<String> = env::args().collect();
        let config = Config::new(&args).unwrap();

        Application { config, router }
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

        let thread_pool = ThreadPool::new(4);

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            scope(|s| {
                s.spawn(move |_| {
                    handle_client(stream, &self.router);
                });
            })
            .unwrap();
            // trace!("{}: Handling request", Utc::now().format("%H:%M:%S:%f"));
        }
    }
}

fn setup_middleware() -> Box<dyn Middleware> {
    Box::new(FileMiddleware {
        file_directory: "public",
    })
}

fn create_response(
    s: String,
    middleware: &Box<dyn Middleware>,
    router: &Router,
) -> Result<http::Response, http::Status> {
    let request = http::Request::parse(&s).or(Err(http::Status::BadRequest))?;

    let response = middleware
        .answer(&request)
        .or_else(|_| router.dispatch(&request));

    response
}

fn handle_client(mut stream: TcpStream, router: &Router) {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer[..]).unwrap();

    let req_str = str::from_utf8(&buffer).unwrap();

    let request = String::from(req_str);

    // TODO
    let request_copy = request.clone();
    let first_line = request_copy.splitn(2, "\r\n").next().unwrap();

    let middleware = setup_middleware();

    let response = match create_response(request, &middleware, &router) {
        Ok(r) => {
            info!("{} => {}", first_line, r.status);
            r
        }
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
