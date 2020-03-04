use super::constants::*;
use super::multi_byte::Mb32;
use super::textdata::TextData;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct NameDef<'a>(TextData<'a>);

impl<'a> DecodeBinXml<'a> for NameDef<'a> {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let text = TextData::<'a>::decode_xml(&mut &buf)?;
        Ok(Self(text))
    }
}

pub struct QNameDef {
    namespaceuri: Mb32,
    prefix: Mb32,
    localname: Mb32,
}

impl<'a> DecodeBinXml<'a> for QNameDef {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let namespaceuri = Mb32::decode_xml(&mut &buf)?;
        let prefix = Mb32::decode_xml(&mut &buf)?;
        let localname = Mb32::decode_xml(&mut &buf)?;

        Ok(Self {
            namespaceuri,
            prefix,
            localname,
        })
    }
}
