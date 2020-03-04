use super::constants::*;
use super::DecodeBinXml;
use crate::Result;

pub enum Version {
    Version1 = 0x01,
    Version2 = 0x02,
}

impl<'a> DecodeBinXml<'a> for Version {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        match buf[0] {
            0x01 => Ok(Version::Version1),
            0x02 => Ok(Version::Version2),
            value => Err(protocol_err!(
                "Unprocessable version number. Expected 1, or 2, but received: {:?}",
                value
            )
            .into()),
        }
    }
}
