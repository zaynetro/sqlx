use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use byteorder::{BigEndian, LittleEndian};

#[derive(Debug)]
pub struct LoginAck {
    interface: u8,
    tds_version: u32,
    program_name: String,
    program_version: Version,
}

impl Decode<'_> for LoginAck {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let interface = buf.get_u8()?;
        let tds_version = buf.get_u32::<LittleEndian>()?;
        let program_name = buf.get_utf16_b_str()?;
        let program_version_major = buf.get_u8()?;
        let program_version_minor = buf.get_u8()?;
        let program_version_build = buf.get_u16::<BigEndian>()?;

        Ok(Self {
            interface,
            tds_version,
            program_name,
            program_version: Version {
                major: program_version_major,
                minor: program_version_minor,
                build: program_version_build,
                sub_build: 0,
            },
        })
    }
}
