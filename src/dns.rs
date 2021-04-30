use dns_lookup::{lookup_addr, lookup_host};
use log::debug;
use std::net::IpAddr;

pub fn hostname_to_ip(addr: &String) -> IpAddr {
    let ip: IpAddr = match addr.parse() {
        Ok(parsed) => parsed,
        Err(_) => {
            debug!("Address is not an IP address, trying to resolve it.");

            match lookup_host(addr) {
                Ok(addrs) => addrs.into_iter().nth(0).unwrap(),
                Err(_) => panic!("Given address is neither an IP nor a resolvable name!"),
            }
        }
    };

    ip
}

pub fn ip_to_hostname(addr: &IpAddr) -> Option<String> {
    match lookup_addr(addr) {
        Ok(hostname) => Some(hostname),
        Err(_) => None,
    }
}
