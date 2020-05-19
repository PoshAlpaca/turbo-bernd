#![feature(test)]
use mime;
use std::{
    io::{Read, Write},
    net::TcpStream,
    str,
    sync::mpsc,
    thread,
};
use turbo_bernd::{
    self, http,
    middleware::{FileMiddleware, Middleware},
    routing::Router,
    Application, Config,
};

#[test]
fn e2e() {
    let mut router = Router::new();

    router.register("/hello", http::Method::Get, |_| {
        http::Response::new(http::Status::Ok).body("Hello, world!", mime::TEXT_PLAIN)
    });

    let config = Config::new(5000);

    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let router = Box::new(router);
        let file_middleware = Box::new(FileMiddleware::new("tests/mock"));
        let middleware: Vec<Box<dyn Middleware>> = vec![router, file_middleware];
        Application::new(middleware).run_graceful(config, rx);
    });

    let url = "localhost:5000";

    // TODO: Fix "\r\n\r\n" being necessary
    let request_1 = "GET /hello HTTP/1.1\r\n\r\n";
    let response_1 = make_request(url, request_1);
    assert!(response_1.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response_1.ends_with("Hello, world!"));

    let request_2 = "GET /test_one/test.html HTTP/1.1\r\n\r\n";
    let response_2 = make_request(url, request_2);
    assert!(response_2.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(response_2.ends_with("</html>\n"));

    // Send terminate signal to application thread
    let _ = tx.send(()).unwrap();
    let _ = handle.join();
}

fn make_request(url: &str, req: &str) -> String {
    let mut stream = TcpStream::connect(url).unwrap();
    stream.write_all(req.as_bytes()).unwrap();

    let mut response = String::new();
    let _ = stream.read_to_string(&mut response).unwrap();

    response
}
