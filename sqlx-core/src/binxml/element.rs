use super::constants::{ELEMENT_TOKEN, END_ELEMENT_TOKEN};
use super::multi_byte::Mb32;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Element {
    qname: Mb32,
    // attrs: Vec<Attribute<'a>>,
    // content: Content<'a>,
}

impl<'a> DecodeBinXml<'a> for Element {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != ELEMENT_TOKEN {
            return Err(protocol_err!("Expected ELEMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let qname = Mb32::decode_xml(&mut &buf)?;

        // let mut attrs = Vec::new();
        // loop {
        //     attrs.push(Attribute::<'a>::decode_xml(&mut &buf)?);
        //     if buf[0] == EndAttributesToken as u8 {
        //         break;
        //     }
        // }

        // let content = Content::<'a>::decode_xml(&mut &buf)?;

        if buf[0] != END_ELEMENT_TOKEN as u8 {
            return Err(protocol_err!("Expected ENDELEMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        Ok(Self {
            qname,
            // attrs,
            // content,
        })
    }
}
