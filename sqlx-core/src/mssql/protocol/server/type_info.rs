use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::collation::Collation;
use crate::mssql::protocol::server::data_type::DataType;
use crate::mssql::protocol::server::xml_info::XmlInfo;
use crate::mssql::protocol::Decode;
use byteorder::{BigEndian, LittleEndian};

#[derive(Debug)]
pub enum TypeInfo {
    FixedLen(DataType),
    VariableLenCollation(TypeInfoWithCollation),
    VariableLenPercision(PreciseTypeInfo),
    VariableLenScale(ScaledTypeInfo),
    VariableLen(DataType),
    PartialLen(PartialLen),
}

impl TypeInfo {
    pub fn has_table_name(&self) -> bool {
        match self {
            TypeInfo::FixedLen(data_type) => data_type.has_table_name(),
            TypeInfo::VariableLenCollation(inner) => inner.data_type.has_table_name(),
            TypeInfo::VariableLenPercision(inner) => inner.data_type.has_table_name(),
            TypeInfo::VariableLenScale(inner) => inner.data_type.has_table_name(),
            TypeInfo::VariableLen(data_type) => data_type.has_table_name(),
            TypeInfo::PartialLen(inner) => inner.data_type.has_table_name(),
            _ => false,
        }
    }
}

#[derive(Debug)]
pub struct TypeInfoWithCollation {
    pub len: u32,
    pub data_type: DataType,
    pub collation: Collation,
}

#[derive(Debug)]
pub struct PreciseTypeInfo {
    pub len: u32,
    pub data_type: DataType,
    pub percision: u8,
    pub scale: u8,
}

#[derive(Debug)]
pub struct ScaledTypeInfo {
    pub len: u32,
    pub data_type: DataType,
    pub scale: u8,
}

#[derive(Debug)]
pub struct PartialLen {
    pub data_type: DataType,
    pub collation: Option<Collation>,
    pub xml_info: Option<XmlInfo>,
    // udt_info: Option<UdtInfo>,
}

impl Decode<'_> for TypeInfo {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let data_type = DataType::decode(buf.get_u8()?)?;
        if data_type.is_fixed_len() {
            Ok(TypeInfo::FixedLen(data_type))
        } else if data_type.is_var_len() {
            let len: u32 = if data_type.is_bytelen() {
                buf.get_u8()? as u32
            } else if data_type.is_ushort_len() {
                buf.get_u16::<LittleEndian>()? as u32
            } else {
                buf.get_u32::<LittleEndian>()?
            };

            Ok(if data_type.has_collation() {
                TypeInfo::VariableLenCollation(TypeInfoWithCollation {
                    len,
                    data_type,
                    collation: Collation::decode(buf)?,
                })
            } else if data_type.has_only_scale() {
                TypeInfo::VariableLenScale(ScaledTypeInfo {
                    len,
                    scale: data_type.scale_to_len(buf.get_u8()?),
                    data_type,
                })
            } else if data_type.has_precision_and_scale() {
                TypeInfo::VariableLenPercision(PreciseTypeInfo {
                    len,
                    percision: buf.get_u8()?,
                    scale: data_type.scale_to_len(buf.get_u8()?),
                    data_type,
                })
            } else {
                TypeInfo::VariableLen(data_type)
            })
        } else if data_type.part_len() {
            if !matches!(data_type, DataType::Xml | DataType::Udt) {
                let shortmaxlen = buf.get_u16::<LittleEndian>()?;
                if shortmaxlen != 0xFFFF {
                    panic!(
                        "Expected USHORTMAXLEN to be 0xFFFF, but received {:?}",
                        shortmaxlen
                    );
                }
            }

            let collation = if data_type.has_collation() {
                Some(Collation::decode(buf)?)
            } else {
                None
            };

            let xml_info = if data_type == DataType::Xml {
                Some(XmlInfo::decode(buf)?)
            } else {
                None
            };

            // let udt_info = if data_type == DataType::Udt {
            //     Some(UdtInfo::decode(buf)?)
            // } else {
            //     None
            // };

            Ok(TypeInfo::PartialLen(PartialLen {
                data_type,
                collation,
                xml_info,
                // udt_info,
            }))
        } else {
            todo!()
        }
    }
}
