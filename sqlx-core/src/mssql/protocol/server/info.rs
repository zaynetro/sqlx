use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::Decode;
use byteorder::LittleEndian;

#[derive(Debug)]
pub struct Info {
    number: u32,
    state: u8,
    class: u8,
    message: String,
    server: String,
    procedure: String,
    line: u32,
}

impl Decode<'_> for Info {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let number = buf.get_u32::<LittleEndian>()?;
        let state = buf.get_u8()?;
        let class = buf.get_u8()?;
        let message = buf.get_utf16_us_str()?;
        let server = buf.get_utf16_b_str()?;
        let procedure = buf.get_utf16_b_str()?;
        let line = buf.get_u32::<LittleEndian>()?;

        Ok(Self {
            number,
            state,
            class,
            message,
            server,
            procedure,
            line,
        })
    }
}
