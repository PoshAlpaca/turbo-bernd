use std::env;
use std::process;
use turbo_bernd::http;
use turbo_bernd::routing::Router;
use turbo_bernd::Config;
use turbo_bernd::middleware::FileMiddleware;

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    let mut router = Router::new();

    router.register("/hello", http::Method::Get, |_| http::Response {
        version: http::Version::OneDotOne,
        status: http::Status::Ok,
        headers: http::Headers { headers: Vec::new() },
        body: "Hello, world!".to_string(),
    });

    let file_middleware = FileMiddleware { file_directory: "public" };

    // we don't return anything in the Ok case so only need to handle Err case
    if let Err(e) = turbo_bernd::run(config, router, Box::new(file_middleware)) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
