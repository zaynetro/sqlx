use super::atomic_value::AtomicValue;
use super::comment::Comment;
use super::constants::{CDATA_END_TOKEN, CDATA_TOKEN};
use super::element::Element;
use super::metadata::Metadata;
use super::pi::Pi;
use super::stand_alone::StandAlone;
use super::textdata::TextData;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct CdSect<'a> {
    cdata: Vec<TextData<'a>>,
}

impl<'a> DecodeBinXml<'a> for CdSect<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let mut cdata = Vec::new();

        loop {
            if buf[0] != CDATA_TOKEN {
                return Err(protocol_err!("Expected CDATA_TOKEN, but found: {:?}", buf[0]).into());
            }

            let temp_buf = &buf[1..];
            let textdata = TextData::<'a>::decode_xml(&mut &temp_buf);
            match textdata {
                Ok(textdata) => {
                    buf = temp_buf;
                    cdata.push(textdata)
                }
                Err(_) => break,
            }
        }

        if cdata.len() < 1 {
            return Err(protocol_err!(
                "CDATA_TOKEN must be followed by at least one TextData, but found none"
            )
            .into());
        }

        if buf[0] != CDATA_END_TOKEN {
            return Err(protocol_err!("Expected CDATA_END_TOKEN, but found: {:?}", buf[0]).into());
        }

        Ok(Self { cdata })
    }
}
