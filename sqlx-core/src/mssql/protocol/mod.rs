// mod login;
pub mod message;
mod packet_header;
mod packet_type;
mod status;
// mod prelogin;

// pub use login::Login;
pub use packet_header::PacketHeader;
pub use packet_type::PacketType;
// pub use prelogin::{Encryption, Prelogin, PreloginOption, Version};
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
