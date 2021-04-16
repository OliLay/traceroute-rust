use log::{debug, error};
use pnet::packet::icmp::IcmpTypes;
use pnet::{
    packet::icmp::IcmpType,
    transport::{icmp_packet_iter, TransportChannelType, TransportReceiver, TransportSender},
};
use std::net::IpAddr;
use std::time::Duration;
use std::time::Instant;

#[derive(PartialEq)]
pub enum ReceiveStatus {
    Timeout,
    Error,
    SuccessContinue,
    SuccessDestinationFound,
}

pub trait TracerouteProtocol {
    fn get_protocol(&self) -> TransportChannelType;

    fn send(&self, tx: &mut TransportSender, dst: IpAddr, current_seq: u16) -> Instant;

    fn get_destination_reached_icmp_type(&self) -> IcmpType;

    fn handle(
        &self,
        mut rx: &mut TransportReceiver,
        dst: IpAddr,
    ) -> (ReceiveStatus, Option<IpAddr>, Option<Instant>) {
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
                let icmp_destination_type = self.get_destination_reached_icmp_type();
                match icmp_type {
                    _ if icmp_type == icmp_destination_type => {
                        if destination_found {
                            (
                                ReceiveStatus::SuccessDestinationFound,
                                Some(addr),
                                Some(time_receive),
                            )
                        } else {
                            (ReceiveStatus::Error, None, None)
                        }
                    }
                    IcmpTypes::TimeExceeded => (
                        ReceiveStatus::SuccessContinue,
                        Some(addr),
                        Some(time_receive),
                    ),
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
        };
    }
}
