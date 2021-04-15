use super::args::Config;
use log::{debug, info, error};
use pnet::transport::{TransportProtocol::Ipv4, TransportReceiver, TransportSender};
use pnet::transport::TransportChannelType::Layer4;
use pnet::transport::{transport_channel, icmp_packet_iter};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::util::checksum;
use pnet::packet::Packet;
use std::net::{IpAddr};
use dns_lookup::lookup_host;
use std::time::{Duration, Instant};

enum ReceiveStatus {
    Timeout,
    Error,
    SuccessContinue,
    SuccessDestinationFound
}

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

pub fn do_traceroute(config: Config) {
    let dst = resolve_address(&config.host);

    println!("traceroute-rust to {} ({}), {} hops max", dst, config.host, config.hops);

    let (mut tx, mut rx) = open_socket();

    let mut current_ttl : u8 = 1;
    let mut current_seq : u16 = 0;
    let mut done = false;

    while !done {
        set_ttl(&mut tx, current_ttl);

        for i in 0..3 {
            let time_send = send_icmp_echo_request(&mut tx, dst, current_seq);
            let first = i == 0;
            
            let (status, reply_addr, time_receive_option) = handle_icmp_packet(&mut rx, dst);

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

fn open_socket() -> (TransportSender, TransportReceiver) {
    let protocol = Layer4(Ipv4(IpNextHeaderProtocols::Icmp));

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

fn send_icmp_echo_request(tx: &mut TransportSender, dst : IpAddr, current_seq : u16) -> Instant {
    let buffer = &mut create_buffer();
    let icmp_packet = create_icmp_echo_request(buffer, current_seq);

    match tx.send_to(icmp_packet, dst) {
        Ok(_) => (),
        Err(e) => panic!("Could not send packet.\n{}", e)
    }

    return Instant::now()
}

fn handle_icmp_packet(mut rx : &mut TransportReceiver, dst : IpAddr) -> (ReceiveStatus, Option<IpAddr>, Option<Instant>) {
    let mut iter = icmp_packet_iter(&mut rx);
    
    return match iter.next_with_timeout(Duration::from_secs(2)) {
        Ok(None) => {
            debug!("Timeout, no answer received.");

            (ReceiveStatus::Timeout, None, None)
        }
        Ok(Some((packet, addr))) => {
            let time_receive = Instant::now();
            let mut destination_found = false;
            if addr == dst {
                debug!("Found destination, stopping");
                destination_found = true
            }
                
            let icmp_type = packet.get_icmp_type();
            match icmp_type {
                IcmpTypes::EchoReply => {
                    if destination_found {
                        (ReceiveStatus::SuccessDestinationFound, Some(addr), Some(time_receive))

                    } else {
                        (ReceiveStatus::Error, None, None)
                    }
                }
                IcmpTypes::TimeExceeded => {
                    (ReceiveStatus::SuccessContinue, Some(addr), Some(time_receive))
                     
                },
                _ => {
                    error!("Received ICMP packet, but type is '{:?}'", icmp_type);
                    (ReceiveStatus::Error, None, None)
                }
            }
        }
        Err(err) => {
            error!("Could not receive packet: {}", err);
            (ReceiveStatus::Error, None, None)
        }
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

fn create_buffer() -> Vec<u8> {
    vec![0; 8]
}

fn create_icmp_echo_request(buffer: &mut Vec<u8>, sequence_number: u16) -> MutableEchoRequestPacket {
    use pnet::packet::icmp::echo_request::IcmpCodes;

    let mut packet = MutableEchoRequestPacket::new(buffer).unwrap();

    packet.set_icmp_type(IcmpTypes::EchoRequest);
    packet.set_icmp_code(IcmpCodes::NoCode);
    packet.set_identifier(1234); //TODO: random identifier at startup
    packet.set_sequence_number(sequence_number);

    let checksum = checksum(&packet.to_immutable().packet(), 1);
    packet.set_checksum(checksum);

    packet
}