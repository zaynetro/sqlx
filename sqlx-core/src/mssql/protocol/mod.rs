pub mod client;
mod message_type;
mod packet_header;
mod packet_type;
pub mod server;
mod status;

pub use message_type::MessageType;
pub use packet_header::PacketHeader;
pub use packet_type::PacketType;
pub use status::Status;

pub trait Encode {
    fn r#type() -> PacketType;
    fn encode(&self, buf: &mut Vec<u8>);
}

pub trait Decode<'de>
where
    Self: Sized,
{
    fn decode(buf: &'de [u8]) -> crate::Result<Self>;
}
