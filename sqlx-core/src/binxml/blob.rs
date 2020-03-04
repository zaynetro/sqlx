use super::constants::*;
use super::multi_byte::{Mb32, Mb64};
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Blob<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for Blob<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}

pub struct Blob64<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for Blob64<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb64::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}
