use log::{debug, error};
use pnet::packet::icmp::IcmpTypes;
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

pub trait TracerouteProtocol {
    fn get_protocol(&self) -> TransportChannelType;

    fn get_rx(&self) -> &mut TransportReceiver;

    fn get_tx(&self) -> &mut TransportSender;

    fn open(&self);

    fn set_ttl(&self, ttl: u8) {
        self.get_tx().set_ttl(ttl);
    }

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

    fn send(&self, dst: IpAddr, current_seq: u16) -> Instant;

    fn get_destination_reached_icmp_type(&self) -> IcmpType;

    fn handle_protocol_level(&self) -> Option<Result> {
        None
    }

    fn handle_icmp_level(&self, dst: IpAddr, wait_secs: u8) -> Result {
        let mut rx = self.get_rx();
        let mut iter = icmp_packet_iter(rx);

        return match iter.next_with_timeout(Duration::from_secs(wait_secs.into())) {
            Ok(None) => {
                debug!("Timeout, no answer received.");

                Result::new_empty(ReceiveStatus::Timeout)
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
                            Result::new_filled(
                                ReceiveStatus::SuccessDestinationFound,
                                addr,
                                time_receive,
                            )
                        } else {
                            Result::new_empty(ReceiveStatus::Error)
                        }
                    }
                    IcmpTypes::TimeExceeded => {
                        Result::new_filled(ReceiveStatus::SuccessContinue, addr, time_receive)
                    }
                    _ => {
                        error!("Received ICMP packet, but type is '{:?}'", icmp_type);
                        Result::new_empty(ReceiveStatus::Error)
                    }
                }
            }
            Err(err) => {
                error!("Could not receive packet: {}", err);
                Result::new_empty(ReceiveStatus::Error)
            }
        };
    }

    fn handle(&self, dst: IpAddr, wait_secs: u8) -> Result {
        match self.handle_protocol_level() {
            Some(result) => result,
            None => self.handle_icmp_level(dst, wait_secs),
        }
    }
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
