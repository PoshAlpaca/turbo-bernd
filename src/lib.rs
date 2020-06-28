#![feature(test)]

use crossbeam::scope;
use log::{debug, error, info};
use std::{
    env,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    str,
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};

pub mod http;
pub mod middleware;
pub mod routing;
pub mod websocket;

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
        let (_tx, rx) = mpsc::channel();

        self.run_graceful(config, rx);
    }

    pub fn run_graceful(&self, config: Config, receiver: Receiver<()>) {
        info!(
            "Starting {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        let address = format!("127.0.0.1:{}", config.port);

        info!("Listening at: {}", address);
        let listener = TcpListener::bind(address).unwrap();
        listener
            .set_nonblocking(true)
            .expect("Could not set listener to non-blocking");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    debug!("Handling new stream");

                    scope(|s| {
                        s.spawn(move |_| {
                            let client = Client::new(stream, Protocol::Http);
                            client.handle();
                            self.handle_client(stream);
                        });
                    })
                    .unwrap();
                }
                Err(ref e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }

            match receiver.try_recv() {
                Ok(_) => {
                    info!("Shutting down");
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    debug!("Application's termination channel disconnected");
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }

    pub fn respond_to(&self, req: &http::Request) -> Response {
        self.dispatch_to_middleware(&req)
            .unwrap_or_else(|e| match e {
                middleware::Error::MethodNotAllowed => Response::new(Status::MethodNotAllowed),
                middleware::Error::NotFound => Response::new(Status::NotFound),
            })
    }

    pub fn respond_to_str(&self, req_str: &str) -> Response {
        match http::Request::parse(req_str) {
            Ok(req) => self.respond_to(&req),
            Err(e) => match e {
                http::Error::UnsupportedVersion => Response::new(Status::VersionNotSupported),
                http::Error::UnknownMethod => Response::new(Status::BadRequest),
                http::Error::MalformedRequest => Response::new(Status::BadRequest),
            },
        }
    }

    fn dispatch_to_middleware(
        &self,
        request: &http::Request,
    ) -> Result<http::Response, middleware::Error> {
        for current in &self.middleware {
            match current.answer(&request) {
                Err(middleware::Error::NotFound) => continue,
                res => return res,
            };
        }

        Err(middleware::Error::NotFound)
    }

    fn handle_client(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer[..]).unwrap();

        let req_str = str::from_utf8(&buffer).unwrap();
        let request = String::from(req_str);

        // TODO
        let request_copy = request.clone();
        let first_line = request_copy.splitn(2, "\r\n").next().unwrap();
        let response = self.respond_to_str(&request);

        let s = format!("{} => {}", first_line, response.status);

        match response.class() {
            ResponseClass::Informational => info!("{}", s),
            ResponseClass::Successful => info!("{}", s),
            ResponseClass::Redirection => info!("{}", s),
            ResponseClass::ClientError => error!("{}", s),
            ResponseClass::ServerError => error!("{}", s),
        }

        let _ = stream.write(format!("{}", response).as_bytes());

        if response.status == Status::SwitchingProtocols {
            self.protocol = Protocol::WebSocket;
        }
    }
}

enum Protocol {
    Http,
    WebSocket,
}

struct Client {
    stream: TcpStream,
    protocol: Protocol,
}

impl Client {
    pub fn new(stream: TcpStream, protocol: Protocol) -> Client {
        Client { stream, protocol }
    }

    pub fn handle(&self) {
        let mut buffer = [0; 1024];
        loop {
            let _ = self.stream.read(&mut buffer[..]).unwrap();

            match self.protocol {
                Protocol::Http => {}
                Protocol::WebSocket => {
                    let mut buffer = [0; 2];
                    let _ = self.stream.read(&mut buffer[..]).unwrap();
                    let bit_vec = websocket::BitVec::from_bytes(&buffer);

                    let frame = websocket::Frame::new(&[], &buffer);

                    let _ = self.stream.write(&frame.as_bytes());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;
    use middleware::MockMiddleware;

    #[test]
    fn mytest() {
        let mut mock = MockMiddleware::new();
        mock.expect_answer()
            .times(1)
            .returning(|_| Ok(Response::new(Status::Ok)));

        let req = Request::get("/hello");

        assert_eq!(Ok(Response::new(Status::Ok)), mock.answer(&req));
    }

    #[test]
    fn dispatch_to_middleware_respects_ordering() {
        let mut mock_a = MockMiddleware::new();
        mock_a
            .expect_answer()
            .times(1)
            .returning(|_| Ok(Response::new(Status::Ok)));

        let mut mock_b = MockMiddleware::new();
        mock_b
            .expect_answer()
            .times(0)
            .returning(|_| Ok(Response::new(Status::Ok)));

        let application = Application::new(vec![Box::new(mock_a), Box::new(mock_b)]);

        let req = Request::get("/");

        let res = application.dispatch_to_middleware(&req);

        assert_eq!(res, Ok(Response::new(Status::Ok)));
    }

    #[test]
    fn dispatch_to_middleware_skips_not_found() {
        let mut mock_a = MockMiddleware::new();
        mock_a
            .expect_answer()
            .times(1)
            .returning(|_| Err(middleware::Error::NotFound));

        let mut mock_b = MockMiddleware::new();
        mock_b
            .expect_answer()
            .times(1)
            .returning(|_| Ok(Response::new(Status::Ok)));

        let application = Application::new(vec![Box::new(mock_a), Box::new(mock_b)]);

        let req = Request::get("/");

        let res = application.dispatch_to_middleware(&req);

        assert_eq!(res, Ok(Response::new(Status::Ok)));
    }

    #[test]
    fn respond_to_returns_error_responses() {
        let application = Application::new(vec![]);

        let req = Request::get("/test");
        let res = application.respond_to(&req);

        assert_eq!(res.status, Status::NotFound);
    }

    #[test]
    fn respond_to_str_returns_error_responses() {
        let application = Application::new(vec![]);

        // TODO: Fix parsing problem with "\r\n\r\n"
        let req_str = "GET /test HTTP/2.0\r\n\r\n";
        let res = application.respond_to_str(req_str);

        assert_eq!(res.status, Status::VersionNotSupported);
    }
}
