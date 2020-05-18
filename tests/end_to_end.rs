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
    self,
    http::{self, Request},
    middleware::{FileMiddleware, Middleware},
    routing::Router,
    Application, Config,
};
extern crate test;
use test::bench::Bencher;

#[bench]
fn request_large_file(b: &mut Bencher) {
    let application = Application::new(vec![]);
    let request = Request::get("/test_two/big_test.html");
    b.iter(|| application.respond_to(&request));
}

#[test]
fn respects_middleware_order() {
    let mut router_a = Router::new();
    router_a.register("/test", http::Method::Get, |_| {
        http::Response::new(http::Status::Ok).body("Hello from Router A", mime::TEXT_PLAIN)
    });

    let mut router_b = Router::new();
    router_b.register("/test", http::Method::Get, |_| {
        http::Response::new(http::Status::Ok).body("Hello from Router B", mime::TEXT_PLAIN)
    });

    let application = Application::new(vec![Box::new(router_a), Box::new(router_b)]);
    let request = Request::get("/test");
    let response = application.respond_to(&request);
    assert_eq!(response.body, "Hello from Router A");
}

#[test]
fn skips_middleware_on_notfound() {
    let router_a = Router::new();

    let mut router_b = Router::new();
    router_b.register("/test", http::Method::Get, |_| {
        http::Response::new(http::Status::Ok).body("Hello from Router B", mime::TEXT_PLAIN)
    });

    let application = Application::new(vec![Box::new(router_a), Box::new(router_b)]);
    let request = Request::get("/test");
    let response = application.respond_to(&request);
    assert_eq!(response.body, "Hello from Router B");
}

#[ignore]
#[test]
fn malformed_http() {
    // turbo_bernd::run();
}

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
