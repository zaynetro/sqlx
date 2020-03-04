use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct SqlDate(u32);

impl<'a> DecodeBinXml<'a> for SqlDate {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let date = ((buf[0] as u32) << 16) & (buf[1] as u32) & ((buf[0] as u32) << 8);
        Ok(Self(date))
    }
}
