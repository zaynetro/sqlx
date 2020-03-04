use super::constants::*;
use super::multi_byte::Mb32;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Extension<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for Extension<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;
        let data = &buf[4..4 + length as usize];
        buf.advance(length as usize);
        Ok(Self(&data))
    }
}
