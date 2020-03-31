use crate::io::Buf;
use crate::io::BufMut;
use crate::mssql::protocol::{PacketType, Status};
use crate::mssql::MsSql;
use bitflags::bitflags;
use byteorder::BigEndian;
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct PacketHeader {
    // Type defines the type of message. Typeis a 1-byte unsigned char. The following table
    // describes the types that are available.
    pub r#type: PacketType,

    // Status is a bit field used to indicate the message state. Statusis a 1-byte unsigned char.
    pub status: Status,

    // Length is the size of the packet including the 8 bytes in the packet header.
    pub length: u16,

    // The process ID on the server, corresponding to the current connection.
    pub server_process_id: u16,

    // Packet ID is used for numbering message packets that contain data in addition to the packet
    // header. Packet ID is a 1-byte, unsigned char. Each time packet data is sent, the value of
    // PacketIDis incremented by 1, modulo 256. This allows the receiver to track the sequence
    // of TDS packets for a given message. This value is currently ignored.
    pub packet_id: u8,

    // This 1 byte is currently not used. This byte SHOULD be set to 0x00 and SHOULD be
    // ignored by the receiver.
    pub window: u8,
}

impl PacketHeader {
    pub fn encode(&self, buf: &mut Vec<u8>) -> usize {
        buf.push(self.r#type as u8);
        buf.push(self.status.bits());

        let offset = buf.len();
        buf.put_u16::<BigEndian>(self.length);

        buf.put_u16::<BigEndian>(self.server_process_id);
        buf.push(self.packet_id);
        buf.push(self.window);

        offset
    }

    pub fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        Ok(Self {
            r#type: PacketType::decode(buf.get_u8()?)?,
            status: Status::from_bits_truncate(buf.get_u8()?),
            length: buf.get_u16::<BigEndian>()?,
            server_process_id: buf.get_u16::<BigEndian>()?,
            packet_id: buf.get_u8()?,
            window: buf.get_u8()?,
        })
    }
}
