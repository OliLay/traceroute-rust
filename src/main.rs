mod args;
mod protocols {
    pub mod icmp;
    pub mod protocol;
    pub mod tcp;
    pub mod udp;
}
mod traceroute;

mod dns;
mod interfaces;

use args::Config;
use protocols::icmp::IcmpTraceroute;
use protocols::protocol::TracerouteProtocol;
use protocols::tcp::TcpTraceroute;
use protocols::udp::UdpTraceroute;

fn main() {
    init_logging();
    let config = parse_config();

    let protocol: Box<dyn TracerouteProtocol> = match config.method {
        args::Method::ICMP => Box::new(IcmpTraceroute::new()),
        args::Method::UDP => Box::new(UdpTraceroute::new(config.port)),
        args::Method::TCP => Box::new(TcpTraceroute::new(config.port)),
    };

    traceroute::do_traceroute(config, protocol);
}

fn init_logging() {
    env_logger::init();
}

fn parse_config() -> Config {
    Config::parse()
}
