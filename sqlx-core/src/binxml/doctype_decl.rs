use super::constants::{DOCTYPE_DECL_TOKEN, PUBLIC_TOKEN, SUBSET_TOKEN, SYSTEM_TOKEN};
use super::stand_alone::StandAlone;
use super::textdata::TextData;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct DoctypeDecl<'a> {
    doctype: TextData<'a>,
    system: Option<TextData<'a>>,
    public: Option<TextData<'a>>,
    subset: Option<TextData<'a>>,
}

impl<'a> DecodeBinXml<'a> for DoctypeDecl<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != DOCTYPE_DECL_TOKEN {
            return Err(protocol_err!("Expected XMLDECL_TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let doctype = TextData::<'a>::decode_xml(&mut &buf[0..])?;

        let system = if buf[0] == SYSTEM_TOKEN {
            buf.advance(1);
            Some(TextData::<'a>::decode_xml(&mut &buf[0..])?)
        } else {
            None
        };

        let public = if buf[0] == PUBLIC_TOKEN {
            buf.advance(1);
            Some(TextData::<'a>::decode_xml(&mut &buf[0..])?)
        } else {
            None
        };

        let subset = if buf[0] == SUBSET_TOKEN {
            buf.advance(1);
            Some(TextData::<'a>::decode_xml(&mut &buf[0..])?)
        } else {
            None
        };

        StandAlone::decode_xml(&mut &buf)?;

        Ok(Self {
            doctype,
            system,
            public,
            subset,
        })
    }
}
