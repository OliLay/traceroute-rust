use super::args::Config;
use super::protocols::{ReceiveStatus, TracerouteProtocol};
use super::dns::{hostname_to_ip, ip_to_hostname};
use log::{error, info};
use std::io;
use std::io::Write;
use std::{net::IpAddr, time::Duration};


pub fn do_traceroute(config: Config, mut protocol: Box<dyn TracerouteProtocol>) {
    let dst = hostname_to_ip(&config.host);

    println!(
        "traceroute-rust to {} ({}), {} hops max",
        dst, config.host, config.hops
    );

    protocol.open();

    let mut current_ttl: u8 = config.first_hop_ttl;
    let mut current_seq: u16 = 0;
    let mut done = false;

    while !done {
        protocol.set_ttl(current_ttl);
        print_ttl(current_ttl);

        let mut prev_reply_addr: Option<IpAddr> = None;
        for _ in 0..config.tries {
            let time_send = protocol.send(dst, current_seq);

            let result = protocol.handle(dst, config.wait_secs);

            match result.status {
                ReceiveStatus::SuccessContinue | ReceiveStatus::SuccessDestinationFound => {
                    let metadata = result.metadata.unwrap();
                    let reply_addr = metadata.addr;
                    let rtt = metadata.time_receive - time_send;

                    match prev_reply_addr {
                        None => print_reply_with_ip(reply_addr, rtt, config.resolve_hostnames),
                        Some(prev_reply_addr) => {
                            if prev_reply_addr == reply_addr {
                                print_reply(rtt)
                            } else {
                                print_reply_with_ip(reply_addr, rtt, config.resolve_hostnames)
                            }
                        }
                    }

                    prev_reply_addr = Some(reply_addr);

                    if result.status == ReceiveStatus::SuccessDestinationFound {
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

fn print_timeout() {
    print!("  *");
    flush_stdout();
}

fn print_ttl(current_ttl: u8) {
    print!("\n  {}", current_ttl);
    flush_stdout();
}

fn print_reply_with_ip(addr: IpAddr, rtt: Duration, resolve_hostnames: bool) {
    if resolve_hostnames {
        let hostname = match ip_to_hostname(&addr) {
            Some(hostname) => hostname,
            None => addr.to_string()
        };
        print!("  {} ({})  {:.3}ms", addr, hostname, duration_to_readable(rtt));
    } else {
        print!("  {}  {:.3}ms", addr, duration_to_readable(rtt));
    }

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
