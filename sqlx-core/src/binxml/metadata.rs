use super::constants::{EXTEN_TOKEN, FLUSH_DEFINED_NAME_TOKENS, NAME_DEF_TOKEN, QNAME_DEF_TOKEN};
use super::extension::Extension;
use super::name::{NameDef, QNameDef};
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub enum Metadata<'a> {
    NameDef(NameDef<'a>),
    QNameDef(QNameDef),
    Extension(Extension<'a>),
    None,
}

impl<'a> DecodeBinXml<'a> for Metadata<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        match buf[0] {
            NAME_DEF_TOKEN => {
                buf.advance(1);
                let name = NameDef::<'a>::decode_xml(&mut &buf)?;
                Ok(Metadata::NameDef(name))
            }

            QNAME_DEF_TOKEN => {
                buf.advance(1);
                let name = QNameDef::decode_xml(&mut &buf)?;
                Ok(Metadata::QNameDef(name))
            }

            EXTEN_TOKEN => {
                buf.advance(1);
                let extension = Extension::<'a>::decode_xml(&mut &buf)?;
                Ok(Metadata::Extension(extension))
            }

            FLUSH_DEFINED_NAME_TOKENS => {
                buf.advance(1);
                Ok(Metadata::None)
            }

            value => {
                return Err(protocol_err!(
                    "Expected namedef, qnamedef, extension, or flush tokens, got: {:?}",
                    value
                )
                .into());
            }
        }
    }
}
