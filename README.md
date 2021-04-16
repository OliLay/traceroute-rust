# traceroute-rust

Simple traceroute application using Rust and `pnet` for Raw socket access.

Currently uses ICMP for traceroutes.

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
    traceroute_rust [OPTIONS] <host>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -m <max-hop>        set maximal hop count [default: 64]
    -M <type>           method ('icmp' or 'udp') for traceroute operations [default: icmp]

ARGS:
    <host>    The host to perform traceroute to.

```

