use std::net::{IpAddr, Ipv4Addr};
use pnet::{datalink::{interfaces, NetworkInterface}};

pub fn get_source_ip() -> Ipv4Addr {
    let all_interfaces = interfaces();
    let considered_interfaces = all_interfaces
        .iter()
        .filter(|iface| !iface.is_loopback() && iface.is_up())
        .collect::<Vec<&NetworkInterface>>();

    let source_ip = considered_interfaces
        .first()
        .unwrap()
        .ips
        .iter()
        .find(|ip| ip.is_ipv4())
        .map(|ip| match ip.ip() {
            IpAddr::V4(ip) => ip,
            _ => panic!("Selected iface for IP src only supports IPv6!"),
        })
        .unwrap();

    source_ip
}

pub fn addr_to_ipv4_addr(addr: IpAddr) -> Ipv4Addr {
    match addr {
        IpAddr::V4(ipv4) => ipv4,
        IpAddr::V6(_) => {
            panic!("No supporto Ipv6o")
        }
    }
}

