use pretty_env_logger;

fn main() {
    pretty_env_logger::init_timed();

    turbo_bernd::run();
}
