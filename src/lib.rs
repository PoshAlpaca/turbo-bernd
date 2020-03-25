use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str;

pub mod http;
pub mod middleware;
pub mod routing;

use middleware::Middleware;
use routing::Router;

pub struct Config {
    pub port: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &str> {
        if args.len() < 2 {
            return Err("too few arguments, please include port");
        }

        let port = args[1].clone();

        Ok(Config { port })
    }
}

// Box<dyn Error> is a pointer to any type that implements Error
// ? unwraps a Result to the value in Ok, if it's an Err then the entire func returns an Err
pub fn run(
    config: Config,
    router: Router,
    middleware: Box<dyn Middleware>,
) -> Result<(), Box<dyn Error>> {
    let address = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(address)?;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let request = http::Request::parse(&handle_client(&stream)?)?;
                let response = match middleware.answer(&request) {
                    Ok(response) => response,
                    Err(_) => router.dispatch(&request)?,
                };

                respond_to_client(&stream, &response);
                println!("{:?}", &response);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    drop(listener);

    Ok(())
}

// "http://www.example.com/hello.txt":

//      GET /hello.txt HTTP/1.1
//      User-Agent: curl/7.16.3 libcurl/7.16.3 OpenSSL/0.9.7l zlib/1.2.3
//      Host: www.example.com
//      Accept-Language: en, mi

//      HTTP/1.1 200 OK
//      Date: Mon, 27 Jul 2009 12:28:53 GMT
//      Server: Apache
//      Last-Modified: Wed, 22 Jul 2009 19:15:56 GMT
//      ETag: "34aa387-d-1568eb00"
//      Accept-Ranges: bytes
//      Content-Length: 51
//      Vary: Accept-Encoding
//      Content-Type: text/plain

//      Hello World! My payload includes a trailing CRLF.

fn handle_client(mut stream: &TcpStream) -> io::Result<String> {
    let mut buffer = [0; 1024];
    let _ = stream.read(&mut buffer[..])?;

    let mystring = str::from_utf8(&buffer).unwrap();
    println!("{}", mystring);

    Ok(String::from(mystring))
}

fn respond_to_client(mut stream: &TcpStream, response: &http::Response) {
    let _ = stream.write(format!("{}", response).as_bytes());
}
