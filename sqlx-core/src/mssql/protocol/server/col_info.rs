use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::type_info::TypeInfo;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Describes the column information in browse mode [MSDN-BROWSE], sp_cursoropen,
//      and sp_cursorfetch.
//
// Token Stream Comments:
//      The token value is 0xA5.
//      The TABNAME token contains the actual table name associated with COLINFO.
//
// The ColInfo element is repeated for each column in the result set.
#[derive(Debug)]
pub struct ColInfo {
    // The actual data length, in bytes, of the ColProperty stream. The length does not include token type and length field
    length: u16,
    properties: Vec<ColProperty>,
}

#[derive(Debug)]
pub struct ColProperty {
    // he column number in the result set.
    col_num: u8,
    // The number of the base table that the column was derived from. The value is 0 if the value of Status is EXPRESSION.
    table_num: u8,
    status: Status,
    // The base column name. This only occurs if DIFFERENT_NAME is set in Status.
    col_name: Option<String>,
}

bitflags! {
    pub struct Status: u8 {
        // The column was not requested, but was added because it was part of a key for the associated table.
        const HIDDEN = 0x10;
        // The column name is different than the requested column name in the case of a column alias.
        const DIFFERENT_NAME = 0x20;
        // The column was the result of an expression.
        const EXPRESSION = 0x40;
        // The column is part of a key for the associated table.
        const KEY = 0x80;
    }
}

impl Decode<'_> for ColProperty {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let col_num = buf.get_u8()?;
        let table_num = buf.get_u8()?;
        let status = Status::from_bits_truncate(buf.get_u8()?);
        let col_name = if status.contains(Status::DIFFERENT_NAME) {
            Some(buf.get_utf16_b_str()?)
        } else {
            None
        };

        Ok(Self {
            col_num,
            table_num,
            status,
            col_name,
        })
    }
}

impl Decode<'_> for ColInfo {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let length = buf.get_u16::<LittleEndian>()?;

        let mut properties = Vec::new();
        for _ in (0..) {
            match ColProperty::decode(buf) {
                Ok(v) => properties.push(v),
                Err(_) => break,
            }
        }

        Ok(Self { length, properties })
    }
}
