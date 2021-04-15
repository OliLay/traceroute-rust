# traceroute-rust

Simple traceroute application using Rust and `pnet` for Raw socket access.

Currently uses ICMP for traceroutes.

## Build
Build with
```bash
cargo build
```

## Usage
```bash
$ traceroute_rust --help
traceroute-rust 
Simple traceroute implementation in Rust using pnet

USAGE:
    traceroute_rust <host> [hops]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <host>    The host to perform traceroute to.
    <hops>    Maximum hops (max. TTL). [default: 64]
```

