#[macro_use]
extern crate afl;
extern crate turbo_bernd;
use turbo_bernd::http::Request;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = Request::parse(&s);
        }
    });
}
