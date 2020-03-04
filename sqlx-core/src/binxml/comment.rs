use super::constants::COMMENT_TOKEN;
use super::textdata::TextData;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Comment<'a>(TextData<'a>);

impl<'a> DecodeBinXml<'a> for Comment<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != COMMENT_TOKEN as u8 {
            return Err(protocol_err!("Expected COMMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let text = TextData::<'a>::decode_xml(&mut &buf)?;

        Ok(Self(text))
    }
}
