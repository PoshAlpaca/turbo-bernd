use crate::http;
use std::collections::HashMap;

type CallbackFunction = fn(&http::Request) -> http::Response;

pub struct Router {
    routes: HashMap<http::Uri, HashMap<http::Method, CallbackFunction>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    pub fn register(&mut self, path: &str, method: http::Method, f: CallbackFunction) {
        let route = self
            .routes
            .entry(http::Uri {
                path: path.to_string(),
            })
            .or_insert(HashMap::new());
        route.insert(method, f);
    }

    pub fn dispatch(&self, req: &http::Request) -> Result<http::Response, &str> {
        match self.routes.get(&req.uri) {
            Some(route) => match route.get(&req.method) {
                Some(f) => Ok(f(req)),
                None => Err("405"),
            },
            None => Err("404"),
        }
    }
}
