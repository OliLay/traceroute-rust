use clap::{App, Arg};

pub enum Method {
    ICMP,
    UDP,
}

pub struct Config {
    pub host: String,
    pub hops: u8,
    pub method: Method,
    pub tries: u8,
    pub wait_secs: u8,
    pub resolve_hostnames: bool,
}

impl Config {
    fn host_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("HOST")
            .takes_value(true)
            .help("The host to perform traceroute to.")
            .required(true)
            .index(1)
    }

    fn hops_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("MAX_HOPS")
            .short("m")
            .long("max-hop")
            .takes_value(true)
            .help("set maximal hop count")
            .default_value("64")
    }

    fn mode_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("METHOD")
            .short("M")
            .long("type")
            .takes_value(true)
            .help("method ('icmp' or 'udp') for traceroute operations")
            .default_value("icmp")
    }

    fn tries_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("TRIES")
            .short("q")
            .long("tries")
            .takes_value(true)
            .help("send TRIES probe packets per hop")
            .default_value("3")
    }

    fn wait_arg<'a, 'b>() -> Arg<'a, 'b> {
        Arg::with_name("WAIT_SECS")
            .short("w")
            .long("wait")
            .takes_value(true)
            .help("wait WAIT_SECS seconds for response")
            .default_value("3")
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
            .arg(Config::tries_arg())
            .arg(Config::wait_arg())
            .arg(Config::resolve_hostnames_arg());

        let matches = app.get_matches();
        let host = matches.value_of("HOST").expect("Please specify a host.");
        let hops = matches.value_of("MAX_HOPS").unwrap();
        let method = match matches.value_of("METHOD").unwrap() {
            "icmp" => Method::ICMP,
            "udp" => Method::UDP,
            _ => panic!("Not an available method."),
        };
        let tries = matches.value_of("TRIES").unwrap();
        let wait_secs = matches.value_of("WAIT_SECS").unwrap();
        let resolve_hostnames = matches.is_present("resolve-hostnames");

        let config = Config {
            host: host.to_string(),
            hops: hops.parse::<u8>().unwrap(),
            method: method,
            tries: tries.parse::<u8>().unwrap(),
            wait_secs: wait_secs.parse::<u8>().unwrap(),
            resolve_hostnames: resolve_hostnames
        };

        config
    }
}
