use super::constants::XML_DECL_TOKEN;
use super::stand_alone::StandAlone;
use super::textdata::TextData;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct XmlDecl<'a> {
    textdata1: TextData<'a>,
    textdata2: Option<TextData<'a>>,
}

impl<'a> DecodeBinXml<'a> for XmlDecl<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != XML_DECL_TOKEN {
            return Err(protocol_err!("Expected XMLDECL_TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let textdata1 = TextData::<'a>::decode_xml(&mut &buf[0..])?;

        let textdata2 = if buf[0] == XML_DECL_TOKEN {
            buf.advance(1);
            Some(TextData::<'a>::decode_xml(&mut &buf[0..])?)
        } else {
            None
        };

        StandAlone::decode_xml(&mut &buf)?;

        Ok(Self {
            textdata1,
            textdata2,
        })
    }
}
