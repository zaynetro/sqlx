use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub enum Sign {
    Positive = 0x01,
    Negative = 0x00,
}

impl<'a> DecodeBinXml<'a> for Sign {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        match buf[0] {
            0 => {
                buf.advance(1);
                Ok(Self::Negative)
            }

            1 => {
                buf.advance(1);
                Ok(Self::Positive)
            }

            _ => Err(protocol_err!("Sign is neither 0 or 1").into()),
        }
    }
}
