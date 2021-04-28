use super::protocols::{MinimumChannels, ReceiveStatus, Result, TracerouteProtocol};
use log::{debug, warn};
use pnet::packet::icmp::IcmpTypes;
use pnet::transport::TransportChannelType::Layer4;
use pnet::{
    packet::ip::IpNextHeaderProtocols,
    transport::{
        TransportChannelType, TransportProtocol::Ipv4, TransportReceiver, TransportSender,
    },
};
use pnet::{
    packet::tcp::{MutableTcpPacket, TcpFlags},
    transport::tcp_packet_iter,
};
use rand::Rng;
use std::time::{Duration, Instant};
use std::net::IpAddr;

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

        //let checksum = ipv4_checksum(&packet.to_immutable(), &src, &dst);
        packet.set_checksum(0);

        packet
    }

    fn create_rst_packet<'packet>(
        &self,
        buffer: &'packet mut Vec<u8>,
    ) -> MutableTcpPacket<'packet> {
        let mut packet = self.create_request(buffer, 0);
        packet.set_flags(TcpFlags::RST);

        packet
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
        /*
         TODO: listen also for TCP messages
          This  method  uses well-known "half-open technique", which prevents ap-
         plications on the destination host from seeing our probes at all.  Nor-
         mally,  a tcp syn (DONE!) is sent. For non-listened ports we receive tcp reset,
         and all is done. For active listening ports we receive tcp syn+ack, but
         answer  by tcp reset (instead of expected tcp ack), this way the remote
         tcp session is dropped even without the application ever taking notice.
        */
        Layer4(Ipv4(IpNextHeaderProtocols::Tcp))
    }

    fn send(&mut self, dst: IpAddr, current_seq: u16) -> Instant {
        let mut buffer = self.create_buffer();
        let tcp_packet = self.create_request(&mut buffer, current_seq);

        self.get_tx().send_to(tcp_packet, dst).unwrap();

        return Instant::now();
    }

    fn get_destination_reached_icmp_type(&self) -> pnet::packet::icmp::IcmpType {
        IcmpTypes::DestinationUnreachable
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

                    warn!("\naddr_tcp_rx: {}, port_tcp_rx {}\n", addr, packet.get_destination());
                    if addr == dst {
                        let flags = packet.get_flags();
    
                        if flags == TcpFlags::SYN | TcpFlags::ACK {
                            debug!("Received SYN and ACK, sending RST. (half-open)");
                            
                            let mut buffer = self.create_buffer();
                            let rst_packet = self.create_rst_packet(&mut buffer);
                            self.get_tx().send_to(rst_packet, addr).unwrap();
                        } else if flags == TcpFlags::RST {
                            debug!("Received RST, no need to send RST myself.")
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
                    warn!("Received packet not addressed to me but port {}", packet.get_destination());
                    None
                }
            }
            Err(_) => None
        };
    }
}
