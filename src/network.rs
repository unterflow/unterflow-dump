use pnet::datalink::{self, EthernetDataLinkReceiver, EthernetDataLinkSender};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::Packet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::tcp::TcpPacket;
use std::fmt;
use std::net::IpAddr;

#[derive(Debug, PartialEq)]
pub struct CapturedPacket {
    source: Host,
    target: Host,
    sequence: u32,
    payload: Vec<u8>,
}

impl CapturedPacket {
    pub fn len(&self) -> usize {
        self.payload.len()
    }

    pub fn has_port(&self, ports: &[u16]) -> bool {
        ports.contains(&self.source.port) || ports.contains(&self.target.port)
    }

    pub fn payload(&self) -> &[u8] {
        self.payload.as_slice()
    }
}

impl fmt::Display for CapturedPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} -> {} ({} bytes; seq: {})",
            self.source,
            self.target,
            self.len(),
            self.sequence
        )
    }
}

#[derive(Debug, PartialEq)]
struct Host {
    address: IpAddr,
    port: u16,
}


impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}


pub fn list_interfaces() {
    for interface in datalink::interfaces() {
        println!("{}", interface.name);
        debug!("{:?}", interface);
    }
}

pub fn channel_for_interface(
    name: Option<&str>,
) -> Result<(Box<EthernetDataLinkSender>, Box<EthernetDataLinkReceiver>), String> {
    let interface = if let Some(name) = name {
        datalink::interfaces()
            .into_iter()
            .find(|interface| interface.name == *name)
            .ok_or_else(|| format!("Unable to find interface for name: {}", name))?
    } else {
        info!("No interface specified. Selecting first interface found.");
        datalink::interfaces().into_iter().next().ok_or_else(
            || "Unable to find any interface",
        )?
    };

    info!("Opening channel for interface: {}", interface.name);
    debug!("{:?}", interface);

    match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => Ok((tx, rx)),
        Ok(_) => Err("Unsupported channel type".to_string()),
        Err(e) => Err(format!("Unable to create channel: {}", e)),
    }
}

pub fn read_packet(packet: &EthernetPacket) -> Option<CapturedPacket> {
    match packet.get_ethertype() {
        EtherTypes::Ipv4 => read_ipv4_packet(packet),
        EtherTypes::Ipv6 => read_ipv6_packet(packet),
        _ => None,
    }
}

fn read_ipv4_packet(packet: &EthernetPacket) -> Option<CapturedPacket> {
    match Ipv4Packet::new(packet.payload()) {
        Some(header) => {
            read_transport_packet(
                header.get_next_level_protocol(),
                IpAddr::V4(header.get_source()),
                IpAddr::V4(header.get_destination()),
                header.payload(),
            )
        }
        _ => None,
    }
}

fn read_ipv6_packet(packet: &EthernetPacket) -> Option<CapturedPacket> {
    match Ipv6Packet::new(packet.payload()) {
        Some(header) => {
            read_transport_packet(
                header.get_next_header(),
                IpAddr::V6(header.get_source()),
                IpAddr::V6(header.get_destination()),
                header.payload(),
            )
        }
        _ => None,
    }
}

fn read_transport_packet(
    protocol: IpNextHeaderProtocol,
    source: IpAddr,
    destination: IpAddr,
    packet: &[u8],
) -> Option<CapturedPacket> {
    match protocol {
        IpNextHeaderProtocols::Tcp => read_tcp_packet(source, destination, packet),
        _ => None,
    }
}

fn read_tcp_packet(source: IpAddr, destination: IpAddr, packet: &[u8]) -> Option<CapturedPacket> {
    match TcpPacket::new(packet) {
        Some(tcp) => {
            let data_offset = (tcp.get_data_offset() * 4) as usize;
            Some(CapturedPacket {
                source: Host {
                    address: source,
                    port: tcp.get_source(),
                },
                target: Host {
                    address: destination,
                    port: tcp.get_destination(),
                },
                sequence: tcp.get_sequence(),
                payload: packet[data_offset..].to_vec(),
            })
        }
        None => None,
    }
}
