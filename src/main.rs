mod traceroute;
mod args;

use args::Config;


fn main() {
    init_logging();
    let config = parse_config();

    traceroute::do_traceroute(config);
}

fn init_logging() {
    env_logger::init();
}

fn parse_config() -> Config {
    let args = Config::parse();
    args
}
