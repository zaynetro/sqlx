use super::attribute::Attribute;
use super::constants::{ELEMENT_TOKEN, END_ATTRIBUTES_TOKEN, END_ELEMENT_TOKEN};
use super::content::Content;
use super::multi_byte::Mb32;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Element<'a> {
    qname: Mb32,
    attrs: Vec<Attribute<'a>>,
    content: Content<'a>,
}

impl<'a> DecodeBinXml<'a> for Element<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != ELEMENT_TOKEN {
            return Err(protocol_err!("Expected ELEMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let qname = Mb32::decode_xml(&mut &buf)?;

        let mut attrs = Vec::new();
        loop {
            let temp_buf = buf;
            let attr = Attribute::<'a>::decode_xml(&mut &temp_buf);
            match attr {
                Ok(attr) => {
                    buf = temp_buf;
                    attrs.push(attr);
                }

                Err(_) => break,
            }
        }

        if attrs.len() < 1 {
            return Err(protocol_err!(
                "Attributes must have a least one attribute, but none were found"
            )
            .into());
        }

        if buf[0] != END_ATTRIBUTES_TOKEN {
            return Err(protocol_err!(
                "Attributes must be followed by END_ATTRIBUTES_TOKEN, but found token: {:?}",
                buf[0]
            )
            .into());
        }
        buf.advance(1);

        let content = Content::<'a>::decode_xml(&mut &buf)?;

        if buf[0] != END_ELEMENT_TOKEN as u8 {
            return Err(protocol_err!("Expected ENDELEMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        Ok(Self {
            qname,
            attrs,
            content,
        })
    }
}
