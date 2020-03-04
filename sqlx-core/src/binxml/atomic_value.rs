use super::blob::*;
use super::codepage_text::*;
use super::constants;
use super::decimal::*;
use super::multi_byte::*;
use super::sql_date::*;
use super::textdata::TextData;
use super::textdata::*;
use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;
use byteorder::{BigEndian, ByteOrder, LittleEndian};

pub enum AtomicValue<'a> {
    SqlSmallInt(i16),
    SqlInt(i32),
    SqlReal(f32),
    SqlFloat(f64),
    SqlMoney(i64),
    SqlBit(u8),
    SqlTinyInt(i8),
    SqlBigInt(i64),
    SqlUuid([u8; 16]),
    SqlDecimal(Decimal<'a>),
    SqlNumeric(Decimal<'a>),
    SqlBinary(Blob<'a>),
    SqlChar(CodepageText<'a>),
    SqlNChar(TextData<'a>),
    SqlVarBinary(Blob64<'a>),
    SqlVarChar(CodepageText64<'a>),
    SqlNVarChar(TextData64<'a>),
    SqlDateTime(u64),
    SqlSmallDateTime(u32),
    SqlSmallMoney(i32),
    SqlText(CodepageText64<'a>),
    SqlImage(Blob64<'a>),
    SqlNText(TextData64<'a>),
    SqlUdt(Blob<'a>),
    XsdTimeOffset(SqlDateTimeOffset),
    XsdDateTimeOffset(SqlDateTimeOffset),
    XsdDateOffset(SqlDateTimeOffset),
    XsdTime2(SqlDateTime2),
    XsdDateTime2(SqlDateTime2),
    XsdDate2(SqlDate),
    XsdTime(u64),
    XsdDateTime(u64),
    XsdDate(u64),
    XsdBinHex(Blob<'a>),
    XsdBase64(Blob<'a>),
    XsdBoolean(u8),
    XsdDecimal(Decimal<'a>),
    XsdByte(u8),
    XsdUnsignedShort(u16),
    XsdUnsignedInt(u32),
    XsdUnsignedLong(u64),
    XsdQName(Mb32),
}

impl<'a> DecodeBinXml<'a> for AtomicValue<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        Ok(match buf.next() {
            constants::SQL_SMALL_INT => {
                let value = BigEndian::read_i16(&buf);
                buf.advance(2);
                AtomicValue::SqlSmallInt(value)
            }
            constants::SQL_INT => {
                let value = BigEndian::read_i32(&buf);
                buf.advance(4);
                AtomicValue::SqlInt(value)
            }
            constants::SQL_REAL => {
                let value = BigEndian::read_i32(&buf);
                buf.advance(4);
                AtomicValue::SqlReal(value as f32)
            }
            constants::SQL_FLOAT => {
                let value = BigEndian::read_i64(&buf);
                buf.advance(8);
                AtomicValue::SqlFloat(value as f64)
            }
            constants::SQL_BIT => AtomicValue::SqlBit(buf.next()),
            constants::SQL_TINY_INT => AtomicValue::SqlTinyInt(buf.next() as i8),
            constants::SQL_BIG_INT => {
                let value = BigEndian::read_i64(&buf);
                buf.advance(8);
                AtomicValue::SqlBigInt(value)
            }
            constants::SQL_DECIMAL => AtomicValue::SqlDecimal(Decimal::<'a>::decode_xml(&buf)?),
            constants::SQL_NUMERIC => AtomicValue::SqlNumeric(Decimal::<'a>::decode_xml(&buf)?),
            constants::SQL_BINARY => AtomicValue::SqlBinary(Blob::<'a>::decode_xml(&buf)?),
            constants::SQL_CHAR => AtomicValue::SqlChar(CodepageText::<'a>::decode_xml(&buf)?),
            constants::SQL_NCHAR => AtomicValue::SqlNChar(TextData::<'a>::decode_xml(&buf)?),
            constants::SQL_VAR_BINARY => AtomicValue::SqlVarBinary(Blob64::<'a>::decode_xml(&buf)?),
            constants::SQL_VAR_CHAR => {
                AtomicValue::SqlVarChar(CodepageText64::<'a>::decode_xml(&buf)?)
            }
            constants::SQL_NVAR_CHAR => {
                AtomicValue::SqlNVarChar(TextData64::<'a>::decode_xml(&buf)?)
            }
            constants::SQL_TEXT => AtomicValue::SqlText(CodepageText64::<'a>::decode_xml(&buf)?),
            constants::SQL_IMAGE => AtomicValue::SqlImage(Blob64::<'a>::decode_xml(&buf)?),
            constants::SQL_NTEXT => AtomicValue::SqlNText(TextData64::<'a>::decode_xml(&buf)?),
            constants::SQL_UDT => AtomicValue::SqlUdt(Blob::<'a>::decode_xml(&buf)?),
            constants::SQL_MONEY => {
                let value = LittleEndian::read_i64(&buf);
                buf.advance(8);
                AtomicValue::SqlMoney(value)
            }
            constants::SQL_SMALL_MONEY => {
                let value = LittleEndian::read_i32(&buf);
                buf.advance(4);
                AtomicValue::SqlSmallMoney(value)
            }
            constants::SQL_DATE_TIME => {
                let value = BigEndian::read_u64(&buf);
                buf.advance(8);
                AtomicValue::SqlDateTime(value)
            }
            constants::SQL_SMALL_DATE_TIME => {
                let value = BigEndian::read_u32(&buf);
                buf.advance(4);
                AtomicValue::SqlSmallDateTime(value)
            }
            constants::XSD_BIN_HEX => AtomicValue::XsdBinHex(Blob::<'a>::decode_xml(&buf)?),
            constants::XSD_BASE64 => AtomicValue::XsdBase64(Blob::<'a>::decode_xml(&buf)?),
            constants::XSD_BOOLEAN => AtomicValue::XsdBoolean(buf.next()),
            constants::XSD_DECIMAL => AtomicValue::XsdDecimal(Decimal::<'a>::decode_xml(&buf)?),
            constants::XSD_BYTE => AtomicValue::XsdByte(buf.next()),
            constants::XSD_TIME_OFFSET => {
                AtomicValue::XsdTimeOffset(SqlDateTimeOffset::decode_xml(&buf)?)
            }
            constants::XSD_DATE_TIME_OFFSET => {
                AtomicValue::XsdDateTimeOffset(SqlDateTimeOffset::decode_xml(&buf)?)
            }
            constants::XSD_DATE_OFFSET => {
                AtomicValue::XsdDateOffset(SqlDateTimeOffset::decode_xml(&buf)?)
            }
            constants::XSD_TIME2 => AtomicValue::XsdTime2(SqlDateTime2::decode_xml(&buf)?),
            constants::XSD_DATE_TIME2 => AtomicValue::XsdDateTime2(SqlDateTime2::decode_xml(&buf)?),
            constants::XSD_DATE2 => AtomicValue::XsdDate2(SqlDate::decode_xml(&buf)?),
            constants::XSD_TIME => {
                let value = BigEndian::read_u64(&buf);
                buf.advance(8);
                AtomicValue::XsdTime(value)
            }
            constants::XSD_DATE_TIME => {
                let value = BigEndian::read_u64(&buf);
                buf.advance(8);
                AtomicValue::XsdDateTime(value)
            }
            constants::XSD_UNSIGNED_SHORT => {
                let value = BigEndian::read_u16(&buf);
                buf.advance(2);
                AtomicValue::XsdUnsignedShort(value)
            }
            constants::XSD_UNSIGNED_INT => {
                let value = BigEndian::read_u32(&buf);
                buf.advance(4);
                AtomicValue::XsdUnsignedInt(value)
            }
            constants::XSD_UNSIGNED_LONG => {
                let value = BigEndian::read_u64(&buf);
                buf.advance(8);
                AtomicValue::XsdUnsignedLong(value)
            }
            constants::XSD_QNAME => AtomicValue::XsdQName(Mb32::decode_xml(&buf)?),
            _ => todo!(),
        })
    }
}
