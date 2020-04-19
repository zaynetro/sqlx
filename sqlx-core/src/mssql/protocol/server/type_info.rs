use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::collation::Collation;
use crate::mssql::protocol::server::data_type::DataType;
use crate::mssql::protocol::server::xml_info::XmlInfo;
use crate::mssql::protocol::Decode;
use byteorder::{BigEndian, LittleEndian};

#[derive(Default, Debug)]
pub struct TypeInfo {
    pub len: Option<u32>,
    pub r#type: DataType,
    pub collation: Option<Collation>,
    pub percision: Option<u8>,
    pub scale: Option<u8>,
    pub xml_info: Option<XmlInfo>,
    // udt_info: Option<UdtInfo>,
}

impl Decode<'_> for TypeInfo {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let r#type = DataType::decode(buf.get_u8()?)?;
        if r#type.is_fixed_len() {
            Ok(TypeInfo {
                r#type,
                ..Default::default()
            })
        } else if r#type.is_var_len() {
            let len = Some(if r#type.is_bytelen() {
                buf.get_u8()? as u32
            } else if r#type.is_ushort_len() {
                buf.get_u16::<LittleEndian>()? as u32
            } else {
                buf.get_u32::<LittleEndian>()?
            });

            Ok(if r#type.has_collation() {
                TypeInfo {
                    r#type,
                    len,
                    collation: Some(Collation::decode(buf)?),
                    ..Default::default()
                }
            } else if r#type.has_only_scale() {
                TypeInfo {
                    scale: Some(r#type.scale_to_len(buf.get_u8()?)),
                    r#type,
                    len,
                    ..Default::default()
                }
            } else if r#type.has_precision_and_scale() {
                TypeInfo {
                    percision: Some(buf.get_u8()?),
                    scale: Some(r#type.scale_to_len(buf.get_u8()?)),
                    r#type,
                    len,
                    ..Default::default()
                }
            } else {
                TypeInfo {
                    r#type,
                    len,
                    ..Default::default()
                }
            })
        } else if r#type.part_len() {
            if !matches!(r#type, DataType::Xml | DataType::Udt) {
                let shortmaxlen = buf.get_u16::<LittleEndian>()?;
                if shortmaxlen != 0xFFFF {
                    panic!(
                        "Expected USHORTMAXLEN to be 0xFFFF, but received {:?}",
                        shortmaxlen
                    );
                }
            }

            let collation = if r#type.has_collation() {
                Some(Collation::decode(buf)?)
            } else {
                None
            };

            let xml_info = if r#type == DataType::Xml {
                Some(XmlInfo::decode(buf)?)
            } else {
                None
            };

            // let udt_info = if r#type == DataType::Udt {
            //     Some(UdtInfo::decode(buf)?)
            // } else {
            //     None
            // };

            Ok(TypeInfo {
                r#type,
                collation,
                xml_info,
                ..Default::default() // udt_info,
            })
        } else {
            todo!()
        }
    }
}
