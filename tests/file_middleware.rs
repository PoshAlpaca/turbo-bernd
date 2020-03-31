use turbo_bernd::http;
use turbo_bernd::middleware::FileMiddleware;
use turbo_bernd::middleware::Middleware;

use std::fs::File;
use std::io::prelude::*;

fn create_dummy_request(path: &str) -> http::Request {
    http::Request {
        method: http::Method::Get,
        uri: http::Uri {
            path: path.to_string(),
        },
        version: http::Version::OneDotOne,
        headers: Vec::new(),
        body: "".to_string(),
    }
}

fn create_dummy_response(path: &str) -> http::Response {
    let mut buffer = String::new();

    let _ = File::open(path).unwrap().read_to_string(&mut buffer);

    http::Response {
        status: http::Status::Ok,
        version: http::Version::OneDotOne,
        headers: http::Headers {
            headers: Vec::new(),
        },
        body: buffer,
    }
}

#[test]
fn answer_returns_response_with_directory_listing() {
    let file_middleware = FileMiddleware {
        file_directory: "tests/mock",
    };
    let dummy_request = create_dummy_request("/");

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
    let dummy_request = create_dummy_request("/test_one/test.html");

    let response = file_middleware.answer(&dummy_request);

    assert_eq!(
        response.unwrap().body,
        create_dummy_response("tests/mock/test_one/test.html").body
    );
}

#[test]
fn answer_returns_404() {
    let file_middleware = FileMiddleware {
        file_directory: "tests/mock",
    };
    let dummy_request_file = create_dummy_request("/test_one/wrong.html");
    let dummy_request_dir = create_dummy_request("/test_eight");

    let response_file = file_middleware.answer(&dummy_request_file);
    let response_dir = file_middleware.answer(&dummy_request_dir);

    assert_eq!(response_file, Err(http::Status::NotFound));
    assert_eq!(response_dir, Err(http::Status::NotFound));
}
