use super::constants::{ENDNEST_TOKEN, NEST_TOKEN};
use super::document::Document;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct NestedXml<'a>(Document<'a>);

impl<'a> DecodeBinXml<'a> for NestedXml<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != NEST_TOKEN {
            return Err(protocol_err!("Expected NEST_TOKEN, but found: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let document = Document::<'a>::decode_xml(&buf)?;

        if buf[0] != ENDNEST_TOKEN {
            return Err(protocol_err!("Expected ENDNEST_TOKEN, but found: {:?}", buf[0]).into());
        }
        buf.advance(1);

        Ok(Self(document))
    }
}
