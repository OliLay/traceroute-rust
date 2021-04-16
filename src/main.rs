mod args;
mod protocols;
mod traceroute;

use args::Config;
use protocols::IcmpTraceroute;

fn main() {
    init_logging();
    let config = parse_config();


    let protocol = match config.method {
        args::Method::ICMP => IcmpTraceroute::new()
    };

    traceroute::do_traceroute(config, &protocol);
}

fn init_logging() {
    env_logger::init();
}

fn parse_config() -> Config {
    let args = Config::parse();
    args
}
