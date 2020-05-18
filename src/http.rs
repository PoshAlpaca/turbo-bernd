use mime;
use std::{collections::HashMap, fmt};

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

#[derive(Debug, PartialEq)]
pub enum Error {
    UnsupportedVersion,
    UnknownMethod,
    MalformedRequest,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Error::UnsupportedVersion => "Unsupported version",
            Error::UnknownMethod => "Unknown method",
            Error::MalformedRequest => "Malformed request",
        };

        write!(f, "{}", error)
    }
}

#[derive(Debug, PartialEq)]
pub enum Version {
    OneDotOne,
}

impl Version {
    fn parse(input: &str) -> Result<Self, Error> {
        match input {
            "HTTP/1.1" => Ok(Self::OneDotOne),
            _ => Err(Error::UnsupportedVersion),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let version = match self {
            Self::OneDotOne => "HTTP/1.1",
        };

        write!(f, "{}", version)
    }
}

#[derive(Debug, PartialEq)]
pub enum Status {
    Ok,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    VersionNotSupported,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Self::Ok => "200 OK",
            Self::BadRequest => "400 Bad Request",
            Self::Unauthorized => "401 Unauthorized",
            Self::Forbidden => "402 Forbidden",
            Self::NotFound => "404 Not Found",
            Self::MethodNotAllowed => "405 Method Not Allowed",
            Self::VersionNotSupported => "505 HTTP Version Not Supported",
        };

        write!(f, "{}", status)
    }
}

pub enum ResponseClass {
    Informational,
    Successful,
    Redirection,
    ClientError,
    ServerError,
}

impl ResponseClass {
    fn new(status: &Status) -> ResponseClass {
        match status {
            Status::Ok => Self::Successful,
            Status::BadRequest => Self::ClientError,
            Status::Unauthorized => Self::ClientError,
            Status::Forbidden => Self::ClientError,
            Status::NotFound => Self::ClientError,
            Status::MethodNotAllowed => Self::ClientError,
            Status::VersionNotSupported => Self::ServerError,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
}

impl Method {
    fn parse(input: &str) -> Result<Self, Error> {
        let method = match input {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "CONNECT" => Self::Connect,
            "OPTIONS" => Self::Options,
            "TRACE" => Self::Trace,
            _ => return Err(Error::UnknownMethod),
        };

        Ok(method)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Uri {
    pub path: String,
}

impl Uri {
    pub fn new(path: &str) -> Uri {
        Uri {
            path: path.to_string(),
        }
    }

    fn parse(input: &str) -> Result<Self, Error> {
        Ok(Self {
            path: input.to_string(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    fn new(uri: &str) -> Request {
        Request {
            method: Method::Get,
            uri: Uri::parse(uri).unwrap(),
            version: Version::OneDotOne,
            headers: HashMap::new(),
            body: "".to_string(),
        }
    }

    pub fn get(uri: &str) -> Request {
        let mut req = Self::new(uri);
        req.method = Method::Get;
        req
    }

    pub fn post(uri: &str) -> Request {
        let mut req = Self::new(uri);
        req.method = Method::Post;
        req
    }

    pub fn header(mut self, header: (&str, &str)) -> Request {
        self.headers
            .insert(header.0.to_string(), header.1.to_string());
        self
    }

    pub fn body(mut self, body: &str, mime: mime::Mime) -> Request {
        self = self.header(("Content-Length", &body.len().to_string()));
        self = self.header(("Content-Type", mime.essence_str()));
        self.body = body.to_string();
        self
    }
}

// Enables custom formatting of Header HashMap
struct HeadersDisplay<'a>(&'a HashMap<String, String>);

impl<'a> fmt::Display for HeadersDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for header in self.0 {
            write!(f, "{}: {}\r\n", header.0, header.1)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub version: Version,
    pub status: Status,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}\r\n{}\r\n{}",
            self.version,
            self.status,
            HeadersDisplay(&self.headers),
            self.body
        )
    }
}

impl Response {
    pub fn new(status: Status) -> Response {
        Response {
            version: Version::OneDotOne,
            status: status,
            headers: HashMap::new(),
            body: "".to_string(),
        }
    }

    pub fn header(mut self, header: (&str, &str)) -> Response {
        self.headers
            .insert(header.0.to_string(), header.1.to_string());
        self
    }

    pub fn body(mut self, body: &str, mime: mime::Mime) -> Response {
        self = self.header(("Content-Length", &body.len().to_string()));
        self = self.header(("Content-Type", mime.essence_str()));
        self.body = body.to_string();
        self
    }
}

impl Response {
    pub fn class(&self) -> ResponseClass {
        ResponseClass::new(&self.status)
    }
}

impl Request {
    pub fn parse(string: &str) -> Result<Self, Error> {
        let mut sections = string.splitn(2, "\r\n\r\n");

        let header = sections.next().ok_or(Error::MalformedRequest)?; // header missing
        let body = sections.next().ok_or(Error::MalformedRequest)?.to_string(); // body missing

        let mut lines = header.split_terminator("\r\n");
        let req_line = lines.next().ok_or(Error::MalformedRequest)?; // no request line

        let mut req_line_tokens = req_line.split(' ');

        let method = Method::parse(req_line_tokens.next().ok_or(Error::MalformedRequest)?)?; // no method
        let uri = Uri::parse(req_line_tokens.next().ok_or(Error::MalformedRequest)?)?; // no uri
        let version = Version::parse(req_line_tokens.next().ok_or(Error::MalformedRequest)?)?; // no version

        let headers_str: Vec<&str> = lines.collect();

        let mut headers = HashMap::new();

        for line in headers_str {
            let mut split_header = line.split(':');
            let header_key = split_header.next().ok_or(Error::MalformedRequest)?; // no header key
            let header_value = split_header.next().ok_or(Error::MalformedRequest)?.trim(); // no header value
            headers.insert(header_key.to_string(), header_value.to_string());
        }

        Ok(Request {
            method,
            uri,
            version,
            headers,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

    fn create_dummy_request_string() -> &'static str {
        "GET /hello.txt HTTP/1.1\r\n\
        User-Agent: curl/7.16.3 libcurl/7.16.3 OpenSSL/0.9.7l zlib/1.2.3\r\n\
        Host: www.example.com\r\n\
        Accept-Language: en, mi\r\n\
        \r\n\
        This is the body \r\nof the request.\r\n"
    }

    #[test]
    fn request_parsing() {
        let http_req = create_dummy_request_string();

        let request = Request::parse(http_req).unwrap();

        assert_eq!(request.method, Method::Get);
        assert_eq!(request.uri, Uri::new("/hello.txt"));
        assert_eq!(request.version, Version::OneDotOne);
        let mut headers = HashMap::new();
        headers.insert(
            "User-Agent".to_string(),
            "curl/7.16.3 libcurl/7.16.3 OpenSSL/0.9.7l zlib/1.2.3".to_string(),
        );
        headers.insert("Host".to_string(), "www.example.com".to_string());
        headers.insert("Accept-Language".to_string(), "en, mi".to_string());
        assert_eq!(request.headers, headers);
        assert_eq!(
            request.body,
            "This is the body \r\nof the request.\r\n".to_string()
        );
    }

    #[bench]
    fn request_parsing_bench(b: &mut Bencher) {
        let http_req = create_dummy_request_string();

        b.iter(|| Request::parse(http_req))
    }

    #[test]
    fn method_parsing() {
        assert_eq!(Method::parse("GET"), Ok(Method::Get));
        assert_eq!(Method::parse("HEAD"), Ok(Method::Head));
        assert_eq!(Method::parse("POST"), Ok(Method::Post));
        assert_eq!(Method::parse("PUT"), Ok(Method::Put));
        assert_eq!(Method::parse("DELETE"), Ok(Method::Delete));
        assert_eq!(Method::parse("CONNECT"), Ok(Method::Connect));
        assert_eq!(Method::parse("OPTIONS"), Ok(Method::Options));
        assert_eq!(Method::parse("TRACE"), Ok(Method::Trace));

        assert_eq!(Method::parse("SOMETHING"), Err(Error::UnknownMethod));
    }

    #[test]
    fn version_parsing() {
        assert_eq!(Version::parse("HTTP/1.1"), Ok(Version::OneDotOne));

        assert_eq!(Version::parse("XYZ/1.0"), Err(Error::UnsupportedVersion));
    }

    #[test]
    fn request_building() {
        let get_req = Request::get("/test");
        assert_eq!(get_req.method, Method::Get);
        assert_eq!(get_req.uri.path, "/test");

        let post_req = Request::post("/test");
        assert_eq!(post_req.method, Method::Post);
        assert_eq!(post_req.uri.path, "/test");

        let post_req = post_req.body("Hello, world!", mime::TEXT_PLAIN);
        assert_eq!(post_req.body, "Hello, world!".to_string());
        assert_eq!(post_req.headers.get("Content-Length").unwrap(), "13");
        assert_eq!(post_req.headers.get("Content-Type").unwrap(), "text/plain");
    }

    #[test]
    fn response_building() {
        let response = Response::new(Status::Ok);
        assert_eq!(response.status, Status::Ok);

        let response = response.body("Hello, world!", mime::TEXT_PLAIN);
        assert_eq!(response.body, "Hello, world!".to_string());
        assert_eq!(response.headers.get("Content-Length").unwrap(), "13");
        assert_eq!(response.headers.get("Content-Type").unwrap(), "text/plain");

        let response = response.header(("Hello", "World"));
        assert_eq!(response.headers.get("Hello").unwrap(), "World");
    }

    #[test]
    fn response_formatting() {
        let response = Response::new(Status::Ok)
            .body("Hello, world!", mime::TEXT_PLAIN)
            .header(("Hello", "World!"));

        let response_string = format!("{}", response);

        // This is a workaround because when the header HashMap is serialized
        // the headers don't always end up in the same order
        // TODO: Unfortunately this does not catch all possible errors
        assert!(response_string.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(response_string.contains("Content-Length: 13\r\n"));
        assert!(response_string.contains("Content-Type: text/plain\r\n"));
        assert!(response_string.contains("Hello: World!\r\n"));
        assert!(response_string.ends_with("\r\nHello, world!"));
    }
}
