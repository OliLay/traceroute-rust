use crate::protocols::MinimumChannels;

use super::protocols::TracerouteProtocol;
use pnet::packet::icmp::IcmpTypes;
use pnet::packet::udp::MutableUdpPacket;
use pnet::transport::TransportChannelType::Layer4;
use pnet::{
    packet::ip::IpNextHeaderProtocols,
    transport::{
        TransportChannelType, TransportProtocol::Ipv4, TransportReceiver, TransportSender,
    },
};
use rand::Rng;
use std::net::IpAddr;
use std::time::Instant;

pub struct UdpTraceroute {
    src_port: u16,
    dst_port: u16,
    channels: MinimumChannels,
}

impl UdpTraceroute {
    pub fn new(dst_port: u16) -> Self {
        UdpTraceroute {
            src_port: rand::thread_rng().gen_range(30000..40000),
            dst_port: dst_port,
            channels: MinimumChannels::new(),
        }
    }

    fn create_request<'packet>(&self, buffer: &'packet mut Vec<u8>) -> MutableUdpPacket<'packet> {
        let mut packet = MutableUdpPacket::new(buffer).unwrap();

        packet.set_source(self.src_port);
        packet.set_destination(self.dst_port);
        packet.set_length(17);
        packet.set_checksum(0);

        let payload: [u8; 9] = [b'S', b'U', b'P', b'E', b'R', b'M', b'A', b'N', 0x00];
        packet.set_payload(&payload);

        packet
    }

    fn create_buffer(&self) -> Vec<u8> {
        vec![0; 17]
    }
}

impl TracerouteProtocol for UdpTraceroute {
    fn get_protocol(&self) -> TransportChannelType {
        Layer4(Ipv4(IpNextHeaderProtocols::Udp))
    }

    fn send(&mut self, dst: IpAddr, _current_seq: u16) -> Instant {
        let mut buffer = self.create_buffer();
        let udp_packet = self.create_request(&mut buffer);

        self.get_tx().send_to(udp_packet, dst).unwrap();

        return Instant::now();
    }

    fn get_destination_reached_icmp_type(&self) -> pnet::packet::icmp::IcmpType {
        IcmpTypes::DestinationUnreachable
    }

    fn get_rx(&mut self) -> &mut TransportReceiver {
        self.channels.rx_icmp.as_mut().unwrap()
    }

    fn get_tx(&mut self) -> &mut TransportSender {
        self.channels.tx.as_mut().unwrap()
    }

    fn open(&mut self) {
        let (tx_udp, _, rx_icmp) = self.create_channels();

        self.channels.tx = Some(tx_udp);
        self.channels.rx_icmp = Some(rx_icmp);
    }
}
