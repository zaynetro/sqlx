use super::multi_byte::*;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

// For converting `&[u8]` to `&[u16]` is unsafe and I don't want to deal
// with that right now.
// pub struct TextData<'a>(Cow<'a, str>);
pub struct TextData<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for TextData<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}

pub struct TextData64<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for TextData64<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb64::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}
