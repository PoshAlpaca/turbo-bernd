use crate::http;

use mime;
use std::{fmt, fs::File, io::prelude::*, path::Path};

#[derive(Debug, PartialEq)]
pub enum Error {
    NotFound,
    MethodNotAllowed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = match self {
            Error::NotFound => "Not found",
            Error::MethodNotAllowed => "Method not allowed",
        };

        write!(f, "{}", error)
    }
}

pub trait Middleware: Sync {
    fn answer(&self, request: &http::Request) -> Result<http::Response, Error>;
}

pub struct FileMiddleware<'a> {
    pub file_directory: &'a str,
}

impl<'a> FileMiddleware<'a> {
    pub fn new(file_directory: &str) -> FileMiddleware {
        FileMiddleware { file_directory }
    }
}

impl<'a> Middleware for FileMiddleware<'a> {
    fn answer(&self, request: &http::Request) -> Result<http::Response, Error> {
        let mut buffer = String::new();

        let file_path = format!("{}{}", self.file_directory, request.uri.path);

        let path = Path::new(&file_path);

        if path.is_dir() {
            buffer.push_str("<html>");
            buffer.push_str("<body>");

            // if err: problem reading dir
            let entries = path.read_dir().or(Err(Error::NotFound))?;

            for entry in entries {
                // if err: problem reading dir entry
                let entry = entry.or(Err(Error::NotFound))?;

                // if err: entry not a valid string
                let mut entry_name = entry.file_name().into_string().or(Err(Error::NotFound))?;

                // if err: problem with entry metadata
                if entry.metadata().or(Err(Error::NotFound))?.is_dir() {
                    entry_name.push('/');
                }

                buffer.push_str("<p>");
                buffer.push_str(&format!("<a href=\"{}{}\">", request.uri.path, entry_name));
                buffer.push_str(&entry_name);
                buffer.push_str("</a>");
                buffer.push_str("</p>");
            }

            buffer.push_str("</body>");
            buffer.push_str("</html>");
        } else {
            let _ = match File::open(file_path) {
                Ok(mut f) => f.read_to_string(&mut buffer),
                Err(_) => return Err(Error::NotFound),
            };
        }

        let response = http::Response::new(http::Status::Ok).body(buffer, mime::TEXT_HTML);

        Ok(response)
    }
}
