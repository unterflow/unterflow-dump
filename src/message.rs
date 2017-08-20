use rmpv::{Utf8String, Value};
use rmpv::decode::read_value;
use std::fmt;
use unterflow_protocol::{RequestResponseMessage, SingleRequestMessage, TransportMessage};
use unterflow_protocol::io::{Data, HasData};

struct MessagePack<'a>(&'a Data);

impl<'a> fmt::Debug for MessagePack<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut reader = self.0.as_slice();
        if let Ok(value) = read_value(&mut reader) {
            // assume all message pack data has to be a map
            // to distinguish non message pack data which still
            // can be parsed somehow
            if value.is_map() {
                write!(f, "{}", value)?;

                if let Some(values) = value.as_map() {
                    let payload_key = Value::String(Utf8String::from("payload"));
                    let payload = values.iter().find(|&&(ref key, _)| key == &payload_key);

                    if let Some(&(_, Value::Binary(ref bytes))) = payload {
                        if let Ok(value) = read_value(&mut bytes.as_slice()) {
                            write!(f, ", payload (decoded): {}", value)?;
                        }
                    }
                }

                return Ok(());
            }
        }

        // default debug output
        write!(f, "Data({:?})", self.0)
    }
}

pub struct Message(pub TransportMessage);

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = &self.0;
        match *message {
            TransportMessage::RequestResponse(ref r) => {
                writeln!(f, "{:?}", r.frame_header)?;
                writeln!(f, "{:?}", r.transport_header)?;
                writeln!(f, "{:?}", r.request_header)?;
                writeln!(f, "{:?}", r.message_header)?;
                match r.message {
                    RequestResponseMessage::ControlMessageRequest(ref r) => {
                        writeln!(f, "{:?}", r)?;
                        writeln!(f, "{:?}", MessagePack(r.data()))
                    }
                    RequestResponseMessage::ControlMessageResponse(ref r) => {
                        writeln!(f, "{:?}", r)?;
                        writeln!(f, "{:?}", MessagePack(r.data()))
                    }
                    RequestResponseMessage::ExecuteCommandRequest(ref r) => {
                        writeln!(f, "{:?}", r)?;
                        writeln!(f, "{:?}", MessagePack(r.data()))
                    }
                    RequestResponseMessage::ExecuteCommandResponse(ref r) => {
                        writeln!(f, "{:?}", r)?;
                        writeln!(f, "{:?}", MessagePack(r.data()))
                    }
                }
            }
            TransportMessage::SingleRequest(ref r) => {
                writeln!(f, "{:?}", r.frame_header)?;
                writeln!(f, "{:?}", r.transport_header)?;
                writeln!(f, "{:?}", r.message_header)?;
                match r.message {
                    SingleRequestMessage::SubscribedEvent(ref r) => {
                        writeln!(f, "{:?}", r)?;
                        writeln!(f, "{:?}", MessagePack(r.data()))
                    }
                    SingleRequestMessage::AppendRequest(ref r) => {
                        writeln!(f, "{:?}", r)?;
                        writeln!(f, "{:?}", MessagePack(r.data()))
                    }
                }
            }
            TransportMessage::ControlRequest(ref r) => {
                writeln!(f, "{:?}", r.frame_header)?;
                writeln!(f, "{:?}", r.transport_header)?;
                writeln!(f, "{:?}", r.message)
            }
        }
    }
}
