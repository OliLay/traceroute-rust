# traceroute-rust

Simple traceroute application using Rust and `pnet` for Raw socket access.

Currently supports ICMP, UDP and TCP for traceroutes.

## Build
Build with
```bash
cargo build
```

## Usage
See the `--help` parameter:
```bash
$ ./traceroute_rust --help
traceroute-rust 
Simple traceroute implementation in Rust using pnet

USAGE:
    traceroute_rust [FLAGS] [OPTIONS] <HOST>

FLAGS:
    -h, --help                 Prints help information
        --resolve-hostnames    resolve hostnames
    -V, --version              Prints version information

OPTIONS:
    -f, --first-hop <FIRST_HOP>    set initial hop distance, i.e., time-to-live [default: 1]
    -m, --max-hop <MAX_HOPS>       set maximal hop count [default: 64]
    -M, --type <METHOD>            method ('icmp', 'udp' or 'tcp') for traceroute operations [default: icmp]
    -p, --port <PORT>              use destination PORT port (UDP, TCP) [default: 33434]
    -q, --tries <TRIES>            send TRIES probe packets per hop [default: 3]
    -w, --wait <WAIT_SECS>         wait WAIT_SECS seconds for response [default: 3]

ARGS:
    <HOST>    The host to perform traceroute to.
```

