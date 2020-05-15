use crate::http;
use crate::middleware;
use middleware::Middleware;
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

    pub fn dispatch(&self, req: &http::Request) -> Result<http::Response, middleware::Error> {
        match self.routes.get(&req.uri) {
            Some(route) => match route.get(&req.method) {
                Some(f) => Ok(f(req)),
                None => Err(middleware::Error::MethodNotAllowed),
            },
            None => Err(middleware::Error::NotFound),
        }
    }
}

impl<'a> Middleware for Router {
    fn answer(&self, request: &http::Request) -> Result<http::Response, middleware::Error> {
        self.dispatch(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{Request, Response, Status};
    use mime;

    // fn create_dummy_request() -> Request {
    //     Request::get("/test")
    // }

    fn create_dummy_response() -> Response {
        let x = Response::new(Status::Ok).body("Hello, test!", mime::TEXT_PLAIN);
        x
    }

    #[test]
    fn register() {
        let mut router = Router::new();
        router.register("/test", http::Method::Get, |_| create_dummy_response());

        let dummy_request = Request::get("/test");
        let route = router.routes.get(&dummy_request.uri).unwrap();
        let function = route.get(&dummy_request.method).unwrap();
        let response = function(&dummy_request);

        assert_eq!(response, create_dummy_response());
    }

    #[test]
    fn dispatch_returns_response() {
        let mut router = Router::new();
        router.register("/test", http::Method::Get, |_| create_dummy_response());

        let dummy_request = Request::get("/test");
        let response = router.dispatch(&dummy_request).unwrap();

        assert_eq!(response, create_dummy_response());
    }

    #[test]
    fn dispatch_returns_not_found() {
        let mut router = Router::new();
        router.register("/not_test", http::Method::Get, |_| create_dummy_response());

        let dummy_request = Request::get("/test");
        let result = router.dispatch(&dummy_request);

        assert_eq!(result, Err(middleware::Error::NotFound));
    }

    #[test]
    fn dispatch_returns_method_not_allowed() {
        let mut router = Router::new();
        router.register("/test", http::Method::Post, |_| create_dummy_response());

        let dummy_request = Request::get("/test");
        let result = router.dispatch(&dummy_request);

        assert_eq!(result, Err(middleware::Error::MethodNotAllowed));
    }
}
