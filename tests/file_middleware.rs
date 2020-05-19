use turbo_bernd::http::{self, Request, Response};
use turbo_bernd::middleware::{self, FileMiddleware, Middleware};

use mime;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn answer_returns_response_with_directory_listing() {
    let file_middleware = FileMiddleware {
        file_directory: "tests/mock",
    };
    let dummy_request = Request::get("/");

    let response = file_middleware.answer(&dummy_request).unwrap();

    let directories = ["test_one", "test_two"];
    for entry in &directories {
        assert!(response.body.contains(entry));
    }
}

#[test]
fn answer_returns_response_with_file() {
    let file_middleware = FileMiddleware {
        file_directory: "tests/mock",
    };
    let dummy_request = Request::get("/test_one/test.html");

    let response = file_middleware.answer(&dummy_request);

    let mut buffer = String::new();
    let _ = File::open("tests/mock/test_one/test.html")
        .unwrap()
        .read_to_string(&mut buffer);

    let dummy_response = Response::new(http::Status::Ok).body(&buffer, mime::TEXT_HTML);

    assert_eq!(response.unwrap().body, dummy_response.body);
}

#[test]
fn answer_returns_404() {
    let file_middleware = FileMiddleware {
        file_directory: "tests/mock",
    };
    let dummy_request_file = Request::get("/test_one/wrong.html");
    let dummy_request_dir = Request::get("/test_eight");

    let response_file = file_middleware.answer(&dummy_request_file);
    let response_dir = file_middleware.answer(&dummy_request_dir);

    assert_eq!(response_file, Err(middleware::Error::NotFound));
    assert_eq!(response_dir, Err(middleware::Error::NotFound));
}
