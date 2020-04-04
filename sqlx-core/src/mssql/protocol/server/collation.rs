use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

#[derive(Debug)]
pub struct Collation {
    lcid: u32,
    col_flags: ColFlags,
    version: u8,
    sort_id: u8,
}

bitflags! {
    pub struct ColFlags: u8 {
        const IGNORE_CASE = 0x01;
        const IGNORE_ACCENT = 0x02;
        const IGNORE_WIDTH = 0x04;
        const IGNORE_KANA = 0x08;
        const BINARY = 0x10;
        const BINARY2 = 0x20;
    }
}

impl Decode<'_> for Collation {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let value = buf.get_u32::<BigEndian>()?;
        // LCID is the first 20 BITS
        let lcid = (value & 0xFF_FF_F0_00) >> 12;
        let flags = ((value & 0x00_00_0F_F0) >> 4) as u8;
        let version = (value & 0x00_00_00_0F) as u8;
        let col_flags = ColFlags::from_bits_truncate(flags);
        let sort_id = buf.get_u8()?;

        Ok(Self {
            lcid,
            col_flags,
            version,
            sort_id,
        })
    }
}
