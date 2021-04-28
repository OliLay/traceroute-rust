use super::protocols::TracerouteProtocol;
use super::protocols::MinimumChannels;

use pnet::{packet::icmp::echo_request::MutableEchoRequestPacket, transport::TransportReceiver};
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::Packet;
use pnet::transport::TransportChannelType::Layer4;
use pnet::util::checksum;
use pnet::{
    packet::ip::IpNextHeaderProtocols,
    transport::{TransportChannelType, TransportProtocol::Ipv4, TransportSender},
};
use rand::Rng;
use std::net::IpAddr;
use std::time::Instant;

pub struct IcmpTraceroute {
    identifier: u16,
    channels: MinimumChannels
}

impl IcmpTraceroute {
    pub fn new() -> Self {
        IcmpTraceroute {
            identifier: rand::thread_rng().gen::<u16>(),
            channels: MinimumChannels::new()
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

    fn open(&mut self) {
        let (tx_icmp, _, rx_icmp) = self.create_channels();

        self.channels.tx = Some(tx_icmp);
        self.channels.rx_icmp = Some(rx_icmp);
    }

    fn send(&mut self, dst: IpAddr, current_seq: u16) -> Instant {
        let mut buffer = self.create_buffer();
        let icmp_packet = self.create_request(&mut buffer, current_seq);

        self.get_tx().send_to(icmp_packet, dst).unwrap();

        return Instant::now();
    }

    fn get_destination_reached_icmp_type(&self) -> pnet::packet::icmp::IcmpType {
        IcmpTypes::EchoReply
    }

    fn get_rx(&mut self) -> &mut TransportReceiver {
        self.channels.rx_icmp.as_mut().unwrap()
    }

    fn get_tx(&mut self) -> &mut TransportSender {
        self.channels.tx.as_mut().unwrap()
    }
}
