use super::atomic_value::AtomicValue;
use super::cd_sect::CdSect;
use super::comment::Comment;
use super::element::Element;
use super::metadata::Metadata;
use super::nested_xml::NestedXml;
use super::pi::Pi;
use super::stand_alone::StandAlone;
use super::textdata::TextData;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Content<'a> {
    contents: Vec<ContentType<'a>>,
}

pub enum ContentType<'a> {
    Element(Element<'a>),
    CdSect(CdSect<'a>),
    Pi(Pi<'a>),
    Comment(Comment<'a>),
    AtomicValue(AtomicValue<'a>),
    Metadata(Metadata<'a>),
    NestedXml(NestedXml<'a>),
}

impl<'a> DecodeBinXml<'a> for ContentType<'a> {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        if let Ok(comment) = Comment::decode_xml(&buf) {
            Ok(ContentType::Comment(comment))
        } else if let Ok(pi) = Pi::decode_xml(&buf) {
            Ok(ContentType::Pi(pi))
        } else if let Ok(metadata) = Metadata::decode_xml(&buf) {
            Ok(ContentType::Metadata(metadata))
        } else if let Ok(value) = AtomicValue::decode_xml(&buf) {
            Ok(ContentType::AtomicValue(value))
        } else if let Ok(element) = Element::decode_xml(&buf) {
            Ok(ContentType::Element(element))
        } else {
            Err(
                protocol_err!("Misc expected to decode Comment | Pi | Metadata, but all 3 failed")
                    .into(),
            )
        }
    }
}

impl<'a> DecodeBinXml<'a> for Content<'a> {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let mut contents = Vec::new();

        loop {
            let content = ContentType::decode_xml(&mut &buf);
            match content {
                Ok(content) => contents.push(content),
                Err(_) => break,
            }
        }

        Ok(Self { contents })
    }
}
