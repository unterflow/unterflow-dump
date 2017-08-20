#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate loggerv;

extern crate pnet;
extern crate unterflow_protocol;

extern crate rmpv;


mod cli;
mod message;
mod network;

use message::Message;
use network::{channel_for_interface, list_interfaces, read_packet};
use network::CapturedPacket;

use unterflow_protocol::TransportMessage;
use unterflow_protocol::io::FromBytes;

fn main() {
    let args = cli::app().get_matches();

    loggerv::init_with_verbosity(args.occurrences_of("v") + 1).expect("Unable to read verbosity level");

    if args.is_present("list-interfaces") {
        info!("Listing network interfaces");
        list_interfaces();
        return;
    }

    let interface = args.value_of("interface");
    let (_, mut rx) = match channel_for_interface(interface) {
        Ok(c) => c,
        Err(e) => panic!("Unable to get channel for interface: {}", e),
    };

    let ports = values_t!(args, "port", u16).expect("Unable to read port options");
    info!("Capturing TCP ports: {:?}", ports);

    let mut last = None;
    let same = |last: &Option<CapturedPacket>, packet: &CapturedPacket| match *last {
        Some(ref last) => last == packet,
        _ => false,
    };

    let mut packets = rx.iter();

    while let Ok(packet) = packets.next() {
        if let Some(packet) = read_packet(&packet) {
            if !same(&last, &packet) && packet.len() > 0 && packet.has_port(&ports) {
                {
                    let mut payload = packet.payload();
                    while payload.len() > 0 {
                        let previous_len = payload.len();
                        match TransportMessage::from_bytes(&mut payload) {
                            Ok(message) => {
                                println!("==>  Packet: {}", packet);
                                println!("{}", Message(message));
                            }
                            Err(e) => {
                                error!("Unable to parse packet {:?}: {}", packet, e);
                                if payload.len() == previous_len {
                                    error!(
                                        "Failed to read packet, skipping remaining {} bytes",
                                        payload.len()
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
                last = Some(packet);
            }
        }
    }
}
