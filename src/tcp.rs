use super::protocols::TracerouteProtocol;
use pnet::packet::{icmp::IcmpTypes, tcp::ipv4_checksum};
use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::transport::TransportChannelType::Layer4;
use pnet::{
    packet::ip::IpNextHeaderProtocols,
    transport::{TransportChannelType, TransportProtocol::Ipv4, TransportSender},
};
use std::net::IpAddr;
use std::time::Instant;

pub struct TcpTraceroute {
    port: u16
}

impl TcpTraceroute {
    pub fn new(port: u16) -> Self {
        TcpTraceroute {port: port}
    }

    fn create_request<'packet>(
        &self,
        buffer: &'packet mut Vec<u8>,
        sequence_number: u16,
    ) -> MutableTcpPacket<'packet> {
        let mut packet = MutableTcpPacket::new(buffer).unwrap();

        packet.set_source(20000);
        packet.set_destination(self.port);
        packet.set_sequence(sequence_number.into());
        packet.set_acknowledgement(1);
        packet.set_data_offset(5);
        packet.set_flags(TcpFlags::SYN);
        packet.set_window(0);
        packet.set_urgent_ptr(0);
        
        //let checksum = ipv4_checksum(&packet.to_immutable(), &src, &dst);
        packet.set_checksum(0);

        packet
    }

    fn create_buffer(&self) -> Vec<u8> {
        vec![0; 20]
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

    fn send(&self, tx: &mut TransportSender, dst: IpAddr, current_seq: u16) -> Instant {
        let mut buffer = self.create_buffer();
        let tcp_packet = self.create_request(&mut buffer, current_seq);

        tx.send_to(tcp_packet, dst).unwrap();

        return Instant::now();
    }

    fn get_destination_reached_icmp_type(&self) -> pnet::packet::icmp::IcmpType {
        IcmpTypes::DestinationUnreachable
    }
}
