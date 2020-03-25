use crate::http;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub trait Middleware {
    fn answer(&self, request: &http::Request) -> Result<http::Response, &'static str>;
}

pub struct FileMiddleware<'a> {
    pub file_directory: &'a str,
}

impl<'a> Middleware for FileMiddleware<'a> {
    fn answer(&self, request: &http::Request) -> Result<http::Response, &'static str> {
        let mut buffer = String::new();

        let file_path = format!("{}{}", self.file_directory, request.uri.path);

        let path = Path::new(&file_path);

        if path.is_dir() {
            buffer.push_str("<html>");
            buffer.push_str("<body>");

            for entry in path.read_dir().expect("problem reading dir") {
                let entry = entry.expect("problem reading dir entry");

                let mut entry_name = entry
                    .file_name()
                    .into_string()
                    .expect("entry not valid string");

                if entry
                    .metadata()
                    .expect("problem with entry metadata")
                    .is_dir()
                {
                    entry_name.push('/');
                }

                println!("{}", entry_name);

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
                Err(_) => return Err("404"),
            };
        }

        let content_length =
            http::response::Header::Custom("Content-Length".to_string(), buffer.len().to_string());
        let content_type =
            http::response::Header::Custom("Content-Type".to_string(), "text/html".to_string());

        let response = http::Response {
            version: http::Version::OneDotOne,
            status: http::Status::Ok,
            headers: http::Headers {
                headers: vec![content_length, content_type],
            },
            body: buffer,
        };

        Ok(response)
    }
}
