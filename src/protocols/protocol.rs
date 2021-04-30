use super::icmp::process_icmp_message;
use pnet::transport::transport_channel;
use pnet::transport::TransportChannelType::Layer4;
use pnet::{
    packet::icmp::IcmpType,
    transport::{icmp_packet_iter, TransportChannelType, TransportReceiver, TransportSender},
};
use pnet::{packet::ip::IpNextHeaderProtocols, transport::TransportProtocol::Ipv4};
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

pub struct Result {
    pub status: ReceiveStatus,
    pub metadata: Option<AnswerMetadata>,
}

impl Result {
    pub fn new_empty(status: ReceiveStatus) -> Self {
        Result {
            status: status,
            metadata: None,
        }
    }

    pub fn new_filled(status: ReceiveStatus, addr: IpAddr, time_receive: Instant) -> Self {
        Result {
            status: status,
            metadata: Some(AnswerMetadata {
                addr: addr,
                time_receive: time_receive,
            }),
        }
    }
}

pub struct AnswerMetadata {
    pub addr: IpAddr,
    pub time_receive: Instant,
}
pub struct MinimumChannels {
    pub tx: Option<TransportSender>,
    pub rx_icmp: Option<TransportReceiver>,
}

impl MinimumChannels {
    pub fn new() -> Self {
        MinimumChannels {
            tx: None,
            rx_icmp: None,
        }
    }
}

pub trait TracerouteProtocol {
    fn get_protocol(&self) -> TransportChannelType;

    fn get_rx(&mut self) -> &mut TransportReceiver;

    fn get_tx(&mut self) -> &mut TransportSender;

    fn open(&mut self);

    fn set_ttl(&mut self, ttl: u8) {
        self.get_tx().set_ttl(ttl).unwrap();
    }

    fn send(&mut self, dst: IpAddr, current_seq: u16) -> Instant;

    fn create_channels(&self) -> (TransportSender, TransportReceiver, TransportReceiver) {
        let (tx_protocol, rx_protocol) = match transport_channel(4096, self.get_protocol()) {
            Ok((tx, rx)) => (tx, rx),
            Err(e) => panic!("An error occurred when creating tx/rx channel: {}", e),
        };

        let rx_icmp = match transport_channel(4096, Layer4(Ipv4(IpNextHeaderProtocols::Icmp))) {
            Ok((_, rx)) => rx,
            Err(e) => panic!("An error occurred when creating rx channel: {}", e),
        };

        (tx_protocol, rx_protocol, rx_icmp)
    }

    fn get_destination_reached_icmp_type(&self) -> Option<IcmpType> {
        None
    }

    fn handle_protocol_level(&mut self, _dst: IpAddr) -> Option<Result> {
        None
    }

    fn handle_icmp_level(&mut self, dst: IpAddr) -> Option<Result> {
        let icmp_dest_reached_type = self.get_destination_reached_icmp_type();
        let mut iter = icmp_packet_iter(self.get_rx());

        match iter.next_with_timeout(Duration::from_millis(1)) {
            Ok(None) => None,
            Ok(Some((packet, sender))) => {
                process_icmp_message(packet, sender, dst, icmp_dest_reached_type)
            }
            Err(_) => None,
        }
    }

    fn poll(&mut self, dst: IpAddr, wait_secs: u8) -> Result {
        let time_begin = Instant::now();

        while Instant::now() - time_begin < Duration::from_secs(wait_secs.into()) {
            let result = match self.handle_protocol_level(dst) {
                None => self.handle_icmp_level(dst),
                Some(result) => Some(result),
            };

            if result.is_some() {
                return result.unwrap();
            }
        }

        return Result::new_empty(ReceiveStatus::Timeout);
    }
}
