use crate::io::{BufStream, MaybeTlsStream};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, Copy, Clone)]
pub enum MessageType {
    Info,
    EnvChange,
    Error,
    Done,
    LoginAck,
}

impl MessageType {
    pub(crate) fn try_from_u8(ty: u8) -> crate::Result<Self> {
        Ok(match ty {
            0xaa => MessageType::Error,
            0xab => MessageType::Info,
            0xad => MessageType::LoginAck,
            0xe3 => MessageType::EnvChange,
            0xfd => MessageType::Done,

            _ => {
                return Err(protocol_err!("unknown value `0x{:02x?}` for token type", ty).into());
            }
        })
    }

    // Compute the size of the message
    // Consume data from the stream if the length is encoded in the stream
    pub(crate) fn size(
        &self,
        stream: &mut BufStream<MaybeTlsStream>,
        packet: &mut usize,
    ) -> crate::Result<u16> {
        Ok(match self {
            // Most messages encode the size as an immediate USHORT
            MessageType::Error
            | MessageType::EnvChange
            | MessageType::Info
            | MessageType::LoginAck => {
                let size = stream.buffer().read_u16::<LittleEndian>()?;

                stream.consume(2);
                *packet -= 2;

                size
            }

            MessageType::Done => 12,
        })
    }
}
