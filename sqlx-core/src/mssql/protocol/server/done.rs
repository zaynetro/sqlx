use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use byteorder::{BigEndian, LittleEndian};

#[derive(Debug)]
pub struct Done {
    status: u16, // TODO: bitflags
    cursor_command: u16,
    affected_rows: u64, // NOTE: u32 before TDS 7.2
}

impl Decode<'_> for Done {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let status = buf.get_u16::<LittleEndian>()?;
        let cursor_command = buf.get_u16::<LittleEndian>()?;
        let affected_rows = buf.get_u64::<LittleEndian>()?;

        Ok(Self {
            affected_rows,
            status,
            cursor_command,
        })
    }
}
