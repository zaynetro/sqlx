use super::multi_byte::Mb32;
use super::sign::Sign;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Decimal<'a> {
    byte: u8,
    sign: Sign,
    buf: &'a [u8],
}

impl<'a> DecodeBinXml<'a> for Decimal<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?;

        let byte = buf[0];
        buf.advance(1);

        let sign = Sign::decode_xml(&mut &buf)?;

        Ok(Self {
            byte,
            sign,
            buf: &buf[..length.0 as usize],
        })
    }
}
