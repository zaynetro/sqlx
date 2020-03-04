use super::comment::Comment;
use super::metadata::Metadata;
use super::pi::Pi;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub enum Misc<'a> {
    Comment(Comment<'a>),
    Pi(Pi<'a>),
    Metadata(Metadata<'a>),
}

impl<'a> DecodeBinXml<'a> for Misc<'a> {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        if let Ok(comment) = Comment::decode_xml(&buf) {
            Ok(Misc::Comment(comment))
        } else if let Ok(pi) = Pi::decode_xml(&buf) {
            Ok(Misc::Pi(pi))
        } else if let Ok(metadata) = Metadata::decode_xml(&buf) {
            Ok(Misc::Metadata(metadata))
        } else {
            Err(
                protocol_err!("Misc expected to decode Comment | Pi | Metadata, but all 3 failed")
                    .into(),
            )
        }
    }
}
