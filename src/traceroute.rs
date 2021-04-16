use super::args::Config;
use super::protocols::{ReceiveStatus, TracerouteProtocol};
use dns_lookup::lookup_host;
use log::{debug, error, info};
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::{TransportChannelType, TransportReceiver, TransportSender};
use pnet::{packet::ip::IpNextHeaderProtocols, transport::TransportProtocol::Ipv4};
use std::io;
use std::io::Write;
use std::{net::IpAddr, time::Duration};

fn resolve_address(addr: &String) -> IpAddr {
    let ip: IpAddr = match addr.parse() {
        Ok(parsed) => parsed,
        Err(_) => {
            debug!("Address is not an IP address, trying to resolve it.");

            let resolved_dst = match lookup_host(addr) {
                Ok(addrs) => addrs.into_iter().nth(0).unwrap(),
                Err(_) => panic!("Given address is neither an IP nor an resolvable name!"),
            };

            resolved_dst
        }
    };

    ip
}

pub fn do_traceroute(config: Config, protocol: Box<dyn TracerouteProtocol>) {
    let dst = resolve_address(&config.host);

    println!(
        "traceroute-rust to {} ({}), {} hops max",
        dst, config.host, config.hops
    );

    let (mut tx, mut rx) = open_socket(protocol.get_protocol());

    let mut current_ttl: u8 = 1;
    let mut current_seq: u16 = 0;
    let mut done = false;

    while !done {
        set_ttl(&mut tx, current_ttl);
        print_ttl(current_ttl);

        let mut prev_reply_addr: Option<IpAddr> = None;
        for _ in 0..3 {
            let time_send = protocol.send(&mut tx, dst, current_seq);

            let (status, reply_addr, time_receive_option) = protocol.handle(&mut rx, dst);

            match status {
                ReceiveStatus::SuccessContinue | ReceiveStatus::SuccessDestinationFound => {
                    let reply_addr = reply_addr.unwrap();
                    let rtt = time_receive_option.unwrap() - time_send;

                    match prev_reply_addr {
                        None => print_reply_with_ip(reply_addr, rtt),
                        Some(prev_reply_addr) => {
                            if prev_reply_addr == reply_addr {
                                print_reply(rtt)
                            } else {
                                print_reply_with_ip(reply_addr, rtt)
                            }
                        }
                    }

                    prev_reply_addr = Some(reply_addr);

                    if status == ReceiveStatus::SuccessDestinationFound {
                        done = true;
                    }
                }
                ReceiveStatus::Timeout => print_timeout(),
                ReceiveStatus::Error => {}
            }

            current_seq += 1;
        }

        current_ttl += 1;

        if current_ttl >= config.hops {
            info!("Max. hops reached, stopping.");
            done = true
        }
    }

    print!("\n")
}

fn open_socket(protocol: TransportChannelType) -> (TransportSender, TransportReceiver) {
    let tx = match transport_channel(4096, protocol) {
        Ok((tx, _)) => tx,
        Err(e) => panic!("An error occurred when tx channel: {}", e),
    };

    let rx = match transport_channel(4096, Layer4(Ipv4(IpNextHeaderProtocols::Icmp))) {
        Ok((_, rx)) => rx,
        Err(e) => panic!("An error occurred when rx channel: {}", e),
    };

    (tx, rx)
}

fn set_ttl(tx: &mut TransportSender, current_ttl: u8) {
    match tx.set_ttl(current_ttl) {
        Ok(_) => (),
        Err(e) => panic!("Could not set TTL on outgoing ICMP request.\n{}", e),
    }
}

fn print_timeout() {
    print!("  *");
    flush_stdout();
}

fn print_ttl(current_ttl: u8) {
    print!("\n  {}", current_ttl);
    flush_stdout();
}

fn print_reply_with_ip(addr: IpAddr, rtt: Duration) {
    print!("  {}  {:.3}ms", addr, duration_to_readable(rtt));
    flush_stdout();
}

fn print_reply(rtt: Duration) {
    print!("  {:.3}ms", duration_to_readable(rtt));
    flush_stdout();
}

fn duration_to_readable(duration: Duration) -> f32 {
    duration.as_secs_f32() * 1000.0
}

fn flush_stdout() {
    match io::stdout().flush() {
        Err(_) => error!("Could not flush stdout"),
        Ok(_) => (),
    }
}
