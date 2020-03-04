pub mod atomic_value;
pub mod attribute;
pub mod blob;
pub mod cd_sect;
pub mod codepage_text;
pub mod comment;
pub mod constants;
pub mod content;
pub mod decimal;
pub mod doctype_decl;
pub mod document;
pub mod element;
pub mod extension;
pub mod metadata;
pub mod misc;
pub mod multi_byte;
pub mod name;
pub mod nested_xml;
pub mod pi;
pub mod sign;
pub mod sql_date;
pub mod stand_alone;
pub mod textdata;
pub mod version;
pub mod xml_decl;

pub trait DecodeBinXml<'a>: Sized {
    fn decode_xml(buf: &'a [u8]) -> crate::Result<Self>;
}
