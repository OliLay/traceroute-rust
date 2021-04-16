use log::{debug, error};
use pnet::packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::Packet;
use pnet::transport::TransportChannelType::Layer4;
use pnet::util::checksum;
use pnet::{
    packet::ip::IpNextHeaderProtocols,
    transport::{
        icmp_packet_iter, TransportChannelType, TransportProtocol::Ipv4, TransportReceiver,
        TransportSender,
    },
};
use rand::Rng;
use std::net::IpAddr;
use std::time::{Duration, Instant};

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

    fn handle(
        &self,
        rx: &mut TransportReceiver,
        dst: IpAddr,
    ) -> (ReceiveStatus, Option<IpAddr>, Option<Instant>);
}

pub struct IcmpTraceroute {
    identifier: u16,
}

impl IcmpTraceroute {
    pub fn new() -> Self {
        IcmpTraceroute {
            identifier: rand::thread_rng().gen::<u16>(),
        }
    }

    fn create_request<'packet>(
        &self,
        buffer: &'packet mut Vec<u8>,
        sequence_number: u16,
    ) -> MutableEchoRequestPacket<'packet> {
        use pnet::packet::icmp::echo_request::IcmpCodes;

        let mut packet = MutableEchoRequestPacket::new(buffer).unwrap();

        packet.set_icmp_type(IcmpTypes::EchoRequest);
        packet.set_icmp_code(IcmpCodes::NoCode);
        packet.set_identifier(self.identifier);
        packet.set_sequence_number(sequence_number);

        let checksum = checksum(&packet.to_immutable().packet(), 1);
        packet.set_checksum(checksum);

        packet
    }

    fn create_buffer(&self) -> Vec<u8> {
        vec![0; 8]
    }
}

impl TracerouteProtocol for IcmpTraceroute {
    fn get_protocol(&self) -> TransportChannelType {
        Layer4(Ipv4(IpNextHeaderProtocols::Icmp))
    }

    fn send(&self, tx: &mut TransportSender, dst: IpAddr, current_seq: u16) -> Instant {
        let mut buffer = self.create_buffer();
        let icmp_packet = self.create_request(&mut buffer, current_seq);

        tx.send_to(icmp_packet, dst).unwrap();

        return Instant::now();
    }

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
                match icmp_type {
                    IcmpTypes::EchoReply => {
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
