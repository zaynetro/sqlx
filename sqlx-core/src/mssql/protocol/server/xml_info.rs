use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::Decode;
use byteorder::{BigEndian, LittleEndian};

#[derive(Debug)]
pub struct XmlInfo {
    pub xml_schema: Option<XmlSchema>,
}

#[derive(Debug)]
pub struct XmlSchema {
    pub db_name: String,
    pub owning_schema: String,
    pub xml_schema_collection: String,
}

impl Decode<'_> for XmlInfo {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let schema_present = buf.get_u8()?;
        let xml_schema = if schema_present == 1 {
            Some(XmlSchema::decode(buf)?)
        } else {
            None
        };

        Ok(Self { xml_schema })
    }
}

impl Decode<'_> for XmlSchema {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let db_name = buf.get_utf16_b_str()?;
        let owning_schema = buf.get_utf16_b_str()?;
        let xml_schema_collection = buf.get_utf16_us_str()?;

        Ok(Self {
            db_name,
            owning_schema,
            xml_schema_collection,
        })
    }
}
