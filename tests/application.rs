use turbo_bernd::http::{self, Request, Response};
use turbo_bernd::middleware::{self, FileMiddleware, Middleware};
use turbo_bernd::routing::Router;
use turbo_bernd::Application;

use mime;

use std::fs::File;
use std::io::prelude::*;

#[test]
fn route_querying() {
    let mut router = Router::new();

    router.register("/hello", http::Method::Get, |_| {
        http::Response::new(http::Status::Ok).body("Hello, world!", mime::TEXT_PLAIN)
    });

    let router = Box::new(router);
    let file_middleware = Box::new(FileMiddleware::new("tests/mock"));
    let middleware: Vec<Box<dyn Middleware>> = vec![router, file_middleware];

    let application = Application::new(middleware);

    let req_route = Request::get("/hello");
    let res = application.respond_to(&req_route);
    assert_eq!(res.body, "Hello, world!");
    assert_eq!(res.headers.get("Content-Length").unwrap(), "13");
    assert_eq!(res.headers.get("Content-Type").unwrap(), "text/plain");

    let req_wrong_route = Request::get("unknown");
    let res = application.respond_to(&req_wrong_route);
    assert_eq!(res.status, http::Status::NotFound);
}

#[test]
fn route_and_filename_equal_picks_first() {
    let mut router = Router::new();

    router.register("/test_one/test.html", http::Method::Get, |_| {
        http::Response::new(http::Status::Ok).body("Hello from router!", mime::TEXT_PLAIN)
    });

    let router = Box::new(router);
    let file_middleware = Box::new(FileMiddleware::new("tests/mock"));
    let middleware: Vec<Box<dyn Middleware>> = vec![router, file_middleware];

    let application = Application::new(middleware);

    let req_double_route = Request::get("/test_one/test.html");
    let res = application.respond_to(&req_double_route);

    assert_eq!(res.body, "Hello from router!");
}
