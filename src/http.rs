use std::error::Error;
use std::fmt;

pub mod request {
    #[derive(Debug, PartialEq)]
    pub enum Header {
        CacheControl(String),
        Expect(String),
        Host(String),
        MaxForwards(String),
        Pragma(String),
        Range(String),
        TE(String),

        IfMatch(String),
        IfNoneMatch(String),
        IfModifiedSince(String),
        IfUnmodifiedSince(String),
        IfRange(String),

        Accept(String),
        AcceptCharset(String),
        AcceptEncoding(String),
        AcceptLanguage(String),

        Authorization(String),
        ProxyAuthorization(String),

        From(String),
        Referer(String),
        UserAgent(String),

        Custom(String, String),
    }
}

pub mod response {
    use std::fmt;

    #[derive(Debug, PartialEq)]
    pub enum Header {
        Age(String),
        CacheControl,
        Expires,
        Date,
        Location,
        RetryAfter,
        Vary,
        Warning,
        Custom(String, String),
    }

    impl fmt::Display for Header {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let header = match self {
                Self::Age(s) => ("Age".to_string(), s),
                Self::Custom(k, v) => (k.clone(), v),
                _ => panic!("this header does not support formatting"),
            };

            write!(f, "{}: {}", header.0, header.1)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Headers {
    pub headers: Vec<response::Header>,
}

impl fmt::Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for header in &self.headers {
            write!(f, "{}\r\n", header)?;
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum Version {
    OneDotOne,
}

impl Version {
    fn parse(input: &str) -> Result<Self, &'static str> {
        match input {
            "HTTP/1.1" => Ok(Self::OneDotOne),
            _ => Err("wrong version"),
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
        };

        write!(f, "{}", status)
    }
}

impl Error for Status {}

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
    fn parse(input: &str) -> Result<Self, &'static str> {
        let method = match input {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "DELETE" => Self::Delete,
            "CONNECT" => Self::Connect,
            "OPTIONS" => Self::Options,
            "TRACE" => Self::Trace,
            _ => return Err("wrong method"),
        };

        Ok(method)
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Uri {
    pub path: String,
}

impl Uri {
    fn parse(input: &str) -> Result<Self, &'static str> {
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
    pub headers: Vec<request::Header>,
    pub body: String,
}

#[derive(Debug, PartialEq)]
pub struct Response {
    pub version: Version,
    pub status: Status,
    pub headers: Headers,
    pub body: String,
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}\r\n{}\r\n{}",
            self.version, self.status, self.headers, self.body
        )
    }
}

impl Request {
    pub fn parse(string: &str) -> Result<Self, &'static str> {
        let mut sections = string.splitn(2, "\r\n\r\n");

        let header = sections.next().ok_or("no header")?;
        let body = sections.next().ok_or("no body")?.to_string();

        let mut lines = header.split_terminator("\r\n");
        let req_line = lines.next().ok_or("no request line")?;

        let mut req_line_tokens = req_line.split(' ');

        let method = Method::parse(req_line_tokens.next().ok_or("no method")?)?;
        let uri = Uri::parse(req_line_tokens.next().ok_or("no uri")?)?;
        let version = Version::parse(req_line_tokens.next().ok_or("no version")?)?;

        let headers_str: Vec<&str> = lines.collect();

        let mut headers = Vec::new();

        for line in headers_str {
            let mut split_header = line.split(':');
            let header_key = split_header.next().ok_or("no header key")?;
            let header_value = split_header.next().ok_or("no header value")?.trim();
            let header = parse_header(header_key, header_value)?;
            headers.push(header);
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

fn parse_header(key: &str, value: &str) -> Result<request::Header, &'static str> {
    let value = value.to_string();
    let header = match key {
        "Host" => request::Header::Host(value),
        "Accept-Language" => request::Header::AcceptLanguage(value),
        "User-Agent" => request::Header::UserAgent(value),
        _ => request::Header::Custom(key.to_string(), value),
    };

    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;
    use afl::fuzz;

    #[test]
    fn request_parsing() {
        let http_req = "GET /hello.txt HTTP/1.1\r\n\
                        User-Agent: curl/7.16.3 libcurl/7.16.3 OpenSSL/0.9.7l zlib/1.2.3\r\n\
                        Host: www.example.com\r\n\
                        Accept-Language: en, mi\r\n\
                        \r\n\
                        This is the body \r\nof the request.\r\n";

        assert_eq!(
            Request::parse(http_req),
            Ok(Request {
                method: Method::Get,
                uri: Uri {
                    path: "/hello.txt".to_string()
                },
                version: Version::OneDotOne,
                headers: vec![
                    request::Header::UserAgent(
                        "curl/7.16.3 libcurl/7.16.3 OpenSSL/0.9.7l zlib/1.2.3".to_string()
                    ),
                    request::Header::Host("www.example.com".to_string()),
                    request::Header::AcceptLanguage("en, mi".to_string()),
                ],
                body: "This is the body \r\nof the request.\r\n".to_string(),
            })
        );
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

        assert_eq!(Method::parse("SOMETHING"), Err("wrong method"));
    }

    #[test]
    fn version_parsing() {
        assert_eq!(Version::parse("HTTP/1.1"), Ok(Version::OneDotOne));

        assert_eq!(Version::parse("XYZ/1.0"), Err("wrong version"));
    }

    // #[test]
    // #[ignore]
    // fn fuzzing() {
    //     fuzz!(|data: &[u8]| {
    //         if let Ok(s) = std::str::from_utf8(data) {
    //             let _ = Request::parse(&s);
    //         }
    //     });
    // }
}
