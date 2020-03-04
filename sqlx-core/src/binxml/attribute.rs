use super::atomic_value::AtomicValue;
use super::constants::{ATTRIBUTE_TOKEN, END_ATTRIBUTES_TOKEN};
use super::metadata::Metadata;
use super::multi_byte::Mb32;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub struct Attribute<'a> {
    metadata: Vec<Metadata<'a>>,
    qname: Mb32,
    // meta_or_value: Vec<MetaOrAtomicValue<'a>>
}

pub enum MetaOrAtomicValue<'a> {
    Meta(Metadata<'a>),
    AtomicValue(AtomicValue<'a>),
}

impl<'a> DecodeBinXml<'a> for Attribute<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let mut metadata = Vec::new();
        loop {
            let temp_buf = &buf;
            let meta = Metadata::<'a>::decode_xml(&mut &temp_buf);
            match meta {
                Ok(meta) => {
                    metadata.push(meta);
                    buf = temp_buf;
                }

                Err(_) => {
                    break;
                }
            }
        }

        if buf.next() != ATTRIBUTE_TOKEN {
            return Err(protocol_err!("Expected ATTRIBUTE-TOKEN, got: {:?}", buf[0]).into());
        }

        let qname = Mb32::decode_xml(&mut &buf)?;

        let mut meta_or_value: Vec<MetaOrAtomicValue> = Vec::new();

        loop {
            #[allow(unused_mut, unused_assignments)]
            let temp_buf = &buf;

            let meta = Metadata::<'a>::decode_xml(&temp_buf);

            #[allow(unused_mut, unused_assignments)]
            let mut no_meta = false;

            match meta {
                Ok(meta) => {
                    meta_or_value.push(MetaOrAtomicValue::Meta(meta));
                    buf = temp_buf;
                    continue;
                }

                Err(_) => {
                    no_meta = true;
                }
            }

            let temp_buf = &buf;
            let atomic_value = AtomicValue::<'a>::decode_xml(&mut &temp_buf);
            match atomic_value {
                Ok(atomic_value) => {
                    meta_or_value.push(MetaOrAtomicValue::AtomicValue(atomic_value));
                    buf = temp_buf;
                    continue;
                }

                Err(_) => {
                    if no_meta {
                        break;
                    }
                }
            }
        }

        Ok(Self { metadata, qname })
    }
}
