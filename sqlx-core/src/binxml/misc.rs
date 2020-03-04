use super::comment::Comment;
use super::constants::*;
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
