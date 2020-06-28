use mime;
use pretty_env_logger;
use turbo_bernd::{
    http,
    middleware::{FileMiddleware, Middleware},
    routing::Router,
    // websocket::WebsocketMiddleware,
    Application,
    Config,
};

fn main() {
    pretty_env_logger::init_timed();

    let router = Box::new(setup_router());
    let file_middleware = Box::new(FileMiddleware::new("public"));
    let middleware: Vec<Box<dyn Middleware>> = vec![router, file_middleware];

    // let mut ws_middleware = WebSocketMiddleware

    // ws_middleware.register("/chat", |ws| {
    //     ws.
    // });

    let config = Config::new(5000);
    Application::new(middleware).run(config);
}

fn setup_router() -> Router {
    let mut router = Router::new();

    router.register("/hello", http::Method::Get, |_| {
        http::Response::new(http::Status::Ok).body("Hello, world!", mime::TEXT_PLAIN)
    });

    router.register_ws("/echo", |ws| loop {
        let msg = ws.read();
        ws.write(msg);
    });

    router
}
