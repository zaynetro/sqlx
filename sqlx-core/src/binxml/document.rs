use super::constants::ENCODING;
use super::constants::SIGNATURE;
use super::content::Content;
use super::doctype_decl::DoctypeDecl;
use super::misc::Misc;
use super::version::Version;
use super::xml_decl::XmlDecl;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;
use byteorder::{BigEndian, ByteOrder};

pub struct Document<'a> {
    signature: u16,
    version: Version,
    encoding: u16,
    xml_decl: Option<XmlDecl<'a>>,
    doc_misc: Vec<Misc<'a>>,
    doctype: Option<(DoctypeDecl<'a>, Vec<Misc<'a>>)>,
    content: Content<'a>,
}

impl<'a> DecodeBinXml<'a> for Document<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let signature = BigEndian::read_u16(&buf[0..]);
        buf.advance(2);

        let version = Version::decode_xml(&mut &buf[2..])?;

        let encoding = BigEndian::read_u16(&buf[3..]);
        buf.advance(2);

        let temp_buf = buf;
        let xml_decl = if let Ok(xml_decl) = XmlDecl::decode_xml(&mut &temp_buf) {
            buf = temp_buf;
            Some(xml_decl)
        } else {
            None
        };

        let mut doc_misc = Vec::new();

        loop {
            let temp_buf = buf;
            let m = Misc::decode_xml(&mut &temp_buf);

            match m {
                Ok(m) => {
                    buf = temp_buf;
                    doc_misc.push(m);
                }

                Err(_) => break,
            }
        }

        let temp_buf = buf;
        let mut doctype_misc = Vec::new();

        let doctype = if let Ok(doctype_decl) = DoctypeDecl::decode_xml(&mut &temp_buf) {
            buf = temp_buf;

            loop {
                let temp_buf = buf;
                let m = Misc::decode_xml(&mut &temp_buf);

                match m {
                    Ok(m) => {
                        buf = temp_buf;
                        doctype_misc.push(m);
                    }

                    Err(_) => break,
                }
            }

            Some((doctype_decl, doctype_misc))
        } else {
            None
        };

        let content = Content::decode_xml(&mut &buf)?;

        Ok(Self {
            signature,
            version,
            encoding,
            xml_decl,
            doc_misc,
            doctype,
            content,
        })
    }
}
