use super::constants::*;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Mb32(pub u32);

impl<'a> DecodeBinXml<'a> for Mb32 {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let mut value = 0u32;
        for i in 0..5u32 {
            value += ((buf[0] & 0xF0) as u32) << (7u32 * i);

            buf.advance(1);

            if buf[0] & 0xF0 == 0 {
                break;
            }
        }

        Ok(Self(value))
    }
}

pub struct Mb64(pub u64);

impl<'a> DecodeBinXml<'a> for Mb64 {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let mut value = 0u64;
        for i in 0..9u64 {
            value += ((buf[i as usize] & 0xF0) as u64) << (7u64 * i);

            buf.advance(1);

            if buf[i as usize] & 0xF0 == 0 {
                break;
            }
        }

        Ok(Self(value))
    }
}
