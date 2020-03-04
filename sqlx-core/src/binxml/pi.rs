use super::constants::PI_TOKEN;
use super::multi_byte::Mb32;
use super::textdata::TextData;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;
use byteorder::{BigEndian, ByteOrder};

pub struct Pi<'a> {
    name: u32,
    textdata: TextData<'a>,
}

impl<'a> DecodeBinXml<'a> for Pi<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != PI_TOKEN {
            return Err(protocol_err!("Expected PI-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let name = Mb32::decode_xml(&mut &buf)?.0;

        let textdata = TextData::<'a>::decode_xml(&mut &buf)?;

        Ok(Self { name, textdata })
    }
}
