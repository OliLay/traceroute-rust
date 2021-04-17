use clap::{App, Arg};

pub enum Method {
    ICMP,
    UDP,
}

pub struct Config {
    pub host: String,
    pub hops: u8,
    pub method: Method,
    pub resolve_hostnames: bool,
}

impl Config {
    fn host_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("host")
            .takes_value(true)
            .help("The host to perform traceroute to.")
            .required(true)
            .index(1)
    }

    fn hops_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("max-hop")
            .short("m")
            .takes_value(true)
            .help("set maximal hop count")
            .default_value("64")
    }

    fn mode_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("type")
            .short("M")
            .takes_value(true)
            .help("method ('icmp' or 'udp') for traceroute operations")
            .default_value("icmp")
    }

    fn resolve_hostnames_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("resolve-hostnames")
            .long("resolve-hostnames")
            .help("resolve hostnames")
    }

    pub fn parse() -> Self {
        let app = App::new("traceroute-rust")
            .about("Simple traceroute implementation in Rust using pnet")
            .arg(Config::host_arg())
            .arg(Config::hops_arg())
            .arg(Config::mode_arg())
            .arg(Config::resolve_hostnames_arg());

        let matches = app.get_matches();
        let host = matches.value_of("host").expect("Please specify a host.");
        let hops = matches.value_of("max-hop").unwrap();
        let method = match matches.value_of("type").unwrap() {
            "icmp" => Method::ICMP,
            "udp" => Method::UDP,
            _ => panic!("Not an available method."),
        };
        let resolve_hostnames = matches.is_present("resolve-hostnames");

        let config = Config {
            host: host.to_string(),
            hops: hops.parse::<u8>().unwrap(),
            method: method,
            resolve_hostnames: resolve_hostnames
        };

        config
    }
}
