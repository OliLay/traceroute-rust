use super::protocols::TracerouteProtocol;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::udp::MutableUdpPacket;
use pnet::transport::TransportChannelType::Layer4;
use pnet::{
    packet::ip::IpNextHeaderProtocols,
    transport::{TransportChannelType, TransportProtocol::Ipv4, TransportSender},
};
use std::net::IpAddr;
use std::time::Instant;

pub struct UdpTraceroute {
    port: u16
}

impl UdpTraceroute {
    pub fn new(port: u16) -> Self {
        UdpTraceroute {port: port}
    }

    fn create_request<'packet>(
        &self,
        buffer: &'packet mut Vec<u8>,
        _sequence_number: u16,
    ) -> MutableUdpPacket<'packet> {
        let mut packet = MutableUdpPacket::new(buffer).unwrap();

        packet.set_source(20000);
        packet.set_destination(self.port);
        packet.set_length(8);
        packet.set_checksum(0);

        packet
    }

    fn create_buffer(&self) -> Vec<u8> {
        vec![0; 8]
    }
}

impl TracerouteProtocol for UdpTraceroute {
    fn get_protocol(&self) -> TransportChannelType {
        Layer4(Ipv4(IpNextHeaderProtocols::Udp))
    }

    fn send(&self, tx: &mut TransportSender, dst: IpAddr, current_seq: u16) -> Instant {
        let mut buffer = self.create_buffer();
        let udp_packet = self.create_request(&mut buffer, current_seq);

        tx.send_to(udp_packet, dst).unwrap();

        return Instant::now();
    }

    fn get_destination_reached_icmp_type(&self) -> pnet::packet::icmp::IcmpType {
        IcmpTypes::DestinationUnreachable
    }
}
