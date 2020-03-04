use super::constants::ENCODING;
use super::constants::SIGNATURE;
use super::version::Version;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;
use byteorder::{BigEndian, ByteOrder};

pub struct Document {
    signature: u16,
    version: Version,
    encoding: u16,
}

impl<'a> DecodeBinXml<'a> for Document {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let signature = BigEndian::read_u16(&buf[0..]);
        let version = Version::decode_xml(&buf[2..])?;
        let encoding = BigEndian::read_u16(&buf[3..]);
        buf.advance(5);

        Ok(Self {
            signature,
            version,
            encoding,
        })
    }
}
