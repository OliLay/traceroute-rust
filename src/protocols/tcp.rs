use super::protocol::{MinimumChannels, ReceiveStatus, Result, TracerouteProtocol};
use crate::interfaces::{addr_to_ipv4_addr, get_source_ip};
use log::{debug, warn};
use pnet::{
    packet::{
        ip::IpNextHeaderProtocols,
        tcp::{ipv4_checksum, MutableTcpPacket, TcpFlags},
    },
    transport::{
        tcp_packet_iter, TransportChannelType, TransportChannelType::Layer4,
        TransportProtocol::Ipv4, TransportReceiver, TransportSender,
    },
};
use rand::Rng;
use std::net::IpAddr;
use std::{
    net::Ipv4Addr,
    time::{Duration, Instant},
};

pub struct TcpTraceroute {
    src_port: u16,
    dst_port: u16,
    minimum_channels: MinimumChannels,
    rx_tcp: Option<TransportReceiver>,
}

impl TcpTraceroute {
    pub fn new(dst_port: u16) -> Self {
        TcpTraceroute {
            src_port: rand::thread_rng().gen_range(60000..65535),
            dst_port: dst_port,
            minimum_channels: MinimumChannels::new(),
            rx_tcp: None,
        }
    }

    fn create_request<'packet>(
        &self,
        buffer: &'packet mut Vec<u8>,
        _sequence_number: u16,
        dst: Ipv4Addr,
    ) -> MutableTcpPacket<'packet> {
        let mut packet = MutableTcpPacket::new(buffer).unwrap();

        packet.set_source(self.src_port);
        packet.set_destination(self.dst_port);
        packet.set_sequence(1337);
        packet.set_acknowledgement(0);
        packet.set_data_offset(5);
        packet.set_flags(TcpFlags::SYN);
        packet.set_window(0);
        packet.set_urgent_ptr(0);

        let checksum = ipv4_checksum(&packet.to_immutable(), &get_source_ip(), &dst);
        packet.set_checksum(checksum);

        packet
    }

    fn create_rst_packet<'packet>(
        &self,
        buffer: &'packet mut Vec<u8>,
        dst: Ipv4Addr,
    ) -> MutableTcpPacket<'packet> {
        let mut packet = self.create_request(buffer, 0, dst);
        packet.set_flags(TcpFlags::RST);

        packet
    }

    fn send_rst_packet(&mut self, dst: IpAddr) {
        let mut buffer = self.create_buffer();

        let rst_packet = self.create_rst_packet(&mut buffer, addr_to_ipv4_addr(dst));
        &self.get_tx().send_to(rst_packet, dst).unwrap();
    }

    fn create_buffer(&mut self) -> Vec<u8> {
        vec![0; 20]
    }

    fn get_tcp_rx(&mut self) -> &mut TransportReceiver {
        self.rx_tcp.as_mut().unwrap()
    }
}

impl TracerouteProtocol for TcpTraceroute {
    fn get_protocol(&self) -> TransportChannelType {
        Layer4(Ipv4(IpNextHeaderProtocols::Tcp))
    }

    fn send(&mut self, dst: IpAddr, current_seq: u16) -> Instant {
        let mut buffer = self.create_buffer();

        let tcp_packet = self.create_request(&mut buffer, current_seq, addr_to_ipv4_addr(dst));

        self.get_tx().send_to(tcp_packet, dst).unwrap();

        return Instant::now();
    }

    fn get_rx(&mut self) -> &mut TransportReceiver {
        self.minimum_channels.rx_icmp.as_mut().unwrap()
    }

    fn get_tx(&mut self) -> &mut TransportSender {
        self.minimum_channels.tx.as_mut().unwrap()
    }

    fn open(&mut self) {
        let (tx_tcp, rx_tcp, rx_icmp) = self.create_channels();

        self.minimum_channels.tx = Some(tx_tcp);
        self.minimum_channels.rx_icmp = Some(rx_icmp);
        self.rx_tcp = Some(rx_tcp);
    }

    fn handle_protocol_level(&mut self, dst: IpAddr) -> Option<Result> {
        let src_port = self.src_port;
        let rx = self.get_tcp_rx();
        let mut iter = tcp_packet_iter(rx);

        return match iter.next_with_timeout(Duration::from_millis(1)) {
            Ok(None) => None,
            Ok(Some((packet, addr))) => {
                if packet.get_destination() == src_port {
                    let time_receive = Instant::now();

                    if addr == dst {
                        let flags = packet.get_flags();

                        if flags == TcpFlags::SYN | TcpFlags::ACK {
                            // half-open technique
                            debug!("Received SYN and ACK, sending RST. (half-open)");
                            self.send_rst_packet(dst);
                        }

                        Some(Result::new_filled(
                            ReceiveStatus::SuccessDestinationFound,
                            addr,
                            time_receive,
                        ))
                    } else {
                        warn!("Received unexpected packet {:?}", packet);
                        None
                    }
                } else {
                    warn!(
                        "Received packet not addressed to me but port {}",
                        packet.get_destination()
                    );
                    None
                }
            }
            Err(_) => None,
        };
    }
}
