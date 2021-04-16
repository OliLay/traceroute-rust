use clap::{App, Arg};
use log::debug;

pub struct Config {
    pub host: String,
    pub hops: u8,
}

impl Config {
    pub fn parse() -> Self {
        let app = App::new("traceroute-rust")
            .about("Simple traceroute implementation in Rust using pnet");

        let host_arg = Arg::with_name("host")
            .takes_value(true)
            .help("The host to perform traceroute to.")
            .required(true)
            .index(1);

        let hops_arg = Arg::with_name("max-hop")
            .short("m")
            .takes_value(true)
            .help("set maximal hop count")
            .default_value("64");

        let app = app.arg(host_arg).arg(hops_arg);
        let matches = app.get_matches();
        let host = matches.value_of("host").expect("Please specify a host.");
        let hops = matches.value_of("max-hop").unwrap();

        let config = Config {
            host: host.to_string(),
            hops: hops.parse::<u8>().unwrap(),
        };
        debug!("Using '{}' as host.", config.host);
        debug!("Using '{}' as hops.", config.hops);

        config
    }
}
