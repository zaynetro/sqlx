use super::constants::*;
use super::multi_byte::{Mb32, Mb64};
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;
use byteorder::{BigEndian, ByteOrder};

pub struct CodepageText<'a> {
    codepage: u32,
    buf: &'a [u8],
}

impl<'a> DecodeBinXml<'a> for CodepageText<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;

        let codepage = BigEndian::read_u32(&buf);
        buf.advance(4);

        let data = &buf[4..length as usize];
        buf.advance(length as usize);

        Ok(Self {
            codepage,
            buf: data,
        })
    }
}

pub struct CodepageText64<'a> {
    codepage: u32,
    buf: &'a [u8],
}

impl<'a> DecodeBinXml<'a> for CodepageText64<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb64::decode_xml(&mut &buf)?.0;

        let codepage = BigEndian::read_u32(&buf);
        buf.advance(4);

        let data = &buf[4..length as usize];
        buf.advance(length as usize);

        Ok(Self {
            codepage,
            buf: data,
        })
    }
}
