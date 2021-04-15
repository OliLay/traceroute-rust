use super::protocols::{TracerouteProtocol, ReceiveStatus};
use super::args::Config;
use std::{net::IpAddr, time::Duration};
use log::{debug, info};
use pnet::transport::{TransportChannelType, TransportReceiver, TransportSender};
use pnet::transport::{transport_channel};
use dns_lookup::lookup_host;

fn resolve_address(addr: &String) -> IpAddr {
    let ip : IpAddr = match addr.parse() {
        Ok(parsed) => parsed,
        Err(_) => {
            debug!("Address is not an IP address, trying to resolve it.");

            let resolved_dst = match lookup_host(addr) {
                Ok(addrs) => addrs.into_iter().nth(0).unwrap(),
                Err(_) => panic!("Given address is neither an IP nor an resolvable name!")
            };

            resolved_dst
        }
    };

    ip
}

pub fn do_traceroute(config: Config, protocol: &dyn TracerouteProtocol) {
    let dst = resolve_address(&config.host);

    println!("traceroute-rust to {} ({}), {} hops max", dst, config.host, config.hops);

    let (mut tx, mut rx) = open_socket(protocol.get_protocol());

    let mut current_ttl : u8 = 1;
    let mut current_seq : u16 = 0;
    let mut done = false;

    while !done {
        set_ttl(&mut tx, current_ttl);

        for i in 0..3 {
            let time_send = protocol.send(&mut tx, dst, current_seq);
            let first = i == 0;
            
            let (status, reply_addr, time_receive_option) = protocol.handle(&mut rx, dst);

            match status {
                ReceiveStatus::SuccessContinue => {
                    print_reply(first, current_ttl, time_receive_option.unwrap() - time_send, reply_addr.unwrap())
                }
                ReceiveStatus::SuccessDestinationFound => {
                    done = true;
                    print_reply(first, current_ttl, time_receive_option.unwrap() - time_send, reply_addr.unwrap())
                }
                ReceiveStatus::Timeout => {
                    print_timeout(first, current_ttl)
                }
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
    return match transport_channel(4096, protocol) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => panic!("An error occurred when creating the transport channel: {}", e)
    };
}

fn set_ttl(tx : &mut TransportSender, current_ttl : u8) {
    match tx.set_ttl(current_ttl) {
        Ok(_) => (),
        Err(e) => panic!("Could not set TTL on outgoing ICMP request.\n{}", e)
    }
}

fn print_reply(first: bool, current_ttl : u8, rtt : Duration, addr : IpAddr) {
    if first {
        print!("\n  {}  {}  {:.3}ms", current_ttl, addr, rtt.as_secs_f32() * 1000.0)
    } else {
        print!(" {:.3}ms", rtt.as_secs_f32() * 1000.0);
    }   
}

fn print_timeout(first: bool, current_ttl : u8) {
    if first {
        print!("\n  {}  *  *", current_ttl)
    } else {
        print!("    *")
    }
}
