mod args;
mod icmp;
mod protocols;
mod traceroute;
mod udp;

use args::Config;
use icmp::IcmpTraceroute;
use protocols::TracerouteProtocol;
use udp::UdpTraceroute;

fn main() {
    init_logging();
    let config = parse_config();

    let protocol: Box<dyn TracerouteProtocol> = match config.method {
        args::Method::ICMP => Box::new(IcmpTraceroute::new()),
        args::Method::UDP => Box::new(UdpTraceroute::new()),
    };

    traceroute::do_traceroute(config, protocol);
}

fn init_logging() {
    env_logger::init();
}

fn parse_config() -> Config {
    let args = Config::parse();
    args
}
