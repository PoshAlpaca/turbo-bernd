use pretty_env_logger;
use turbo_bernd::{
    http,
    middleware::{FileMiddleware, Middleware},
    routing::Router,
    Application,
};

fn main() {
    pretty_env_logger::init_timed();

    let router = Box::new(setup_router());
    let file_middleware = Box::new(FileMiddleware::new("public"));
    let middleware: Vec<Box<dyn Middleware>> = vec![router, file_middleware];

    Application::new(middleware).run();
}

fn setup_router() -> Router {
    let mut router = Router::new();

    router.register("/hello", http::Method::Get, |_| http::Response {
        version: http::Version::OneDotOne,
        status: http::Status::Ok,
        headers: http::Headers {
            headers: Vec::new(),
        },
        body: "Hello, world!".to_string(),
    });

    router
}
