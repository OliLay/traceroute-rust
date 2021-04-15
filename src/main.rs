mod traceroute;
mod args;
mod protocols;

use args::Config;


fn main() {
    init_logging();
    let config = parse_config();

    let protocol = protocols::IcmpTraceroute::new();
    traceroute::do_traceroute(config, &protocol);
}

fn init_logging() {
    env_logger::init();
}

fn parse_config() -> Config {
    let args = Config::parse();
    args
}
