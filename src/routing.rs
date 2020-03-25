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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_dummy_request() -> http::Request {
        http::Request {
            method: http::Method::Get,
            uri: http::Uri {
                path: "/test".to_string(),
            },
            version: http::Version::OneDotOne,
            headers: Vec::new(),
            body: "".to_string(),
        }
    }

    fn create_dummy_response() -> http::Response {
        http::Response {
            status: http::Status::Ok,
            version: http::Version::OneDotOne,
            headers: http::Headers {
                headers: Vec::new(),
            },
            body: "Hello, test!".to_string(),
        }
    }

    #[test]
    fn register() {
        let mut router = Router::new();
        router.register("/test", http::Method::Get, |_| create_dummy_response());

        let dummy_request = create_dummy_request();
        let route = router.routes.get(&dummy_request.uri).unwrap();
        let function = route.get(&dummy_request.method).unwrap();
        let response = function(&dummy_request);

        assert_eq!(response, create_dummy_response());
    }

    #[test]
    fn dispatch_returns_response() {
        let mut router = Router::new();
        router.register("/test", http::Method::Get, |_| create_dummy_response());

        let dummy_request = create_dummy_request();
        let response = router.dispatch(&dummy_request).unwrap();

        assert_eq!(response, create_dummy_response());
    }

    #[test]
    fn dispatch_returns_404() {
        let mut router = Router::new();
        router.register("/not_test", http::Method::Get, |_| create_dummy_response());

        let dummy_request = create_dummy_request();
        let result = router.dispatch(&dummy_request);

        assert_eq!(result, Err("404"));
    }

    #[test]
    fn dispatch_returns_405() {
        let mut router = Router::new();
        router.register("/test", http::Method::Post, |_| create_dummy_response());

        let dummy_request = create_dummy_request();
        let result = router.dispatch(&dummy_request);

        assert_eq!(result, Err("405"));
    }
}
