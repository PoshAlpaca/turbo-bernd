use pretty_env_logger;
use turbo_bernd::{http, routing::Router, Application};

fn main() {
    pretty_env_logger::init_timed();

    let router = setup_router();

    Application::new(router).run();
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
