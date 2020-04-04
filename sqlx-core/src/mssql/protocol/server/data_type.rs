use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::Decode;

#[derive(Debug, PartialEq, Eq)]
pub enum DataType {
    // Fixed-Length Data Types
    Null = 0x1F,
    Int1 = 0x30,
    Big = 0x32,
    Int2 = 0x34,
    Int4 = 0x38,
    DateTime4 = 0x3A,
    Float4 = 0x3B,
    Money = 0x3C,
    DateTime = 0x3D,
    Float8 = 0x3E,
    Money4 = 0x7A,

    // Variable Lenght DataTypes
    INT8 = 0x7F,
    Guid = 0x24,
    IntN = 0x26,
    Decimal = 0x37,
    Numeric = 0x3F,
    BitN = 0x68,
    DecimalN = 0x6A,
    NumericN = 0x6C,
    FloatN = 0x6D,
    MoneyN = 0x6E,
    DateTimeN = 0x6F,
    DateN = 0x28,
    TimeN = 0x29,
    DateTime2N = 0x2A,
    DateTimeOffsetN = 0x2B,
    Char = 0x2F,
    VarChar = 0x27,
    Binary = 0x2D,
    VarBinary = 0x25,
    BigVarBinary = 0xA5,
    BigVarChar = 0xA7,
    BigBinary = 0xAD,
    BigChar = 0xAF,
    NVarChar = 0xE7,
    NChar = 0xEF,
    Xml = 0xF1,
    Udt = 0xF0,
    Text = 0x23,
    Image = 0x22,
    NText = 0x63,
    SsVariant = 0x62,
}

impl DataType {
    pub fn decode(value: u8) -> crate::Result<Self> {
        Ok(match value {
            0x1F => DataType::Null,
            0x30 => DataType::Int1,
            0x32 => DataType::Big,
            0x34 => DataType::Int2,
            0x38 => DataType::Int4,
            0x3A => DataType::DateTime4,
            0x3B => DataType::Float4,
            0x3C => DataType::Money,
            0x3D => DataType::DateTime,
            0x3E => DataType::Float8,
            0x7A => DataType::Money4,

            0x24 => DataType::Guid,
            0x26 => DataType::IntN,
            0x37 => DataType::Decimal,
            0x3F => DataType::Numeric,
            0x68 => DataType::BitN,
            0x6A => DataType::DecimalN,
            0x6C => DataType::NumericN,
            0x6D => DataType::FloatN,
            0x6E => DataType::MoneyN,
            0x6F => DataType::DateTimeN,
            0x28 => DataType::DateN,
            0x29 => DataType::TimeN,
            0x2A => DataType::DateTime2N,
            0x2B => DataType::DateTimeOffsetN,
            0x2F => DataType::Char,
            0x27 => DataType::VarChar,
            0x2D => DataType::Binary,
            0x25 => DataType::VarBinary,
            0xA5 => DataType::BigVarBinary,
            0xA7 => DataType::BigVarChar,
            0xAD => DataType::BigBinary,
            0xAF => DataType::BigChar,
            0xE7 => DataType::NVarChar,
            0xEF => DataType::NChar,
            0xF1 => DataType::Xml,
            0xF0 => DataType::Udt,
            0x23 => DataType::Text,
            0x22 => DataType::Image,
            0x63 => DataType::NText,
            0x62 => DataType::SsVariant,

            ty => {
                return Err(protocol_err!("unexpected value {:x} for data type", ty).into());
            }
        })
    }
}

impl DataType {
    pub fn is_fixed_len(&self) -> bool {
        match self {
            DataType::Null => true,
            DataType::Int1 => true,
            DataType::Big => true,
            DataType::Int2 => true,
            DataType::Int4 => true,
            DataType::DateTime4 => true,
            DataType::Float4 => true,
            DataType::Money => true,
            DataType::DateTime => true,
            DataType::Float8 => true,
            DataType::Money4 => true,
            _ => false,
        }
    }

    pub fn is_bytelen(&self) -> bool {
        match self {
            DataType::Guid => true,
            DataType::IntN => true,
            DataType::Decimal => true,
            DataType::Numeric => true,
            DataType::BitN => true,
            DataType::DecimalN => true,
            DataType::NumericN => true,
            DataType::FloatN => true,
            DataType::MoneyN => true,
            DataType::DateTimeN => true,
            DataType::DateN => true,
            DataType::TimeN => true,
            DataType::DateTime2N => true,
            DataType::DateTimeOffsetN => true,
            DataType::Char => true,
            DataType::VarChar => true,
            DataType::Binary => true,
            DataType::VarBinary => true,
            _ => false,
        }
    }

    pub fn is_ushort_len(&self) -> bool {
        match self {
            DataType::BigVarBinary => true,
            DataType::BigVarChar => true,
            DataType::BigBinary => true,
            DataType::BigChar => true,
            DataType::NVarChar => true,
            DataType::NChar => true,
            _ => true,
        }
    }

    pub fn is_long_len(&self) -> bool {
        match self {
            DataType::Xml => true,
            DataType::Text => true,
            DataType::Image => true,
            DataType::NText => true,
            DataType::SsVariant => true,
            _ => false,
        }
    }

    pub fn is_var_len(&self) -> bool {
        self.is_bytelen() || self.is_ushort_len() || self.is_long_len()
    }

    pub fn part_len(&self) -> bool {
        match self {
            DataType::Xml => true,
            DataType::BigVarChar => true,
            DataType::BigVarBinary => true,
            DataType::NVarChar => true,
            DataType::Udt => true,
            _ => false,
        }
    }

    pub fn has_collation(&self) -> bool {
        match self {
            DataType::BigChar => true,
            DataType::BigVarChar => true,
            DataType::Text => true,
            DataType::NText => true,
            DataType::NChar => true,
            DataType::NVarChar => true,
            _ => false,
        }
    }

    pub fn has_precision_and_scale(&self) -> bool {
        match self {
            DataType::Numeric => true,
            DataType::NumericN => true,
            DataType::Decimal => true,
            DataType::DecimalN => true,
            _ => false,
        }
    }

    pub fn has_only_scale(&self) -> bool {
        match self {
            DataType::TimeN => true,
            DataType::DateTime2N => true,
            DataType::DateTimeOffsetN => true,
            _ => false,
        }
    }

    pub fn scale_to_len(&self, scale: u8) -> u8 {
        let scale = match scale {
            1 | 2 => 3,
            3 | 4 => 4,
            5 | 6 | 7 => 5,
            _ => panic!(
                "Urecognized scale value. Valid values are 0-7, received {:?}",
                scale
            ),
        };

        let add = match self {
            DataType::TimeN => 0,
            DataType::DateTime2N => 3,
            DataType::DateTimeOffsetN => 5,

            _ => panic!("Attempting to convert scale to length for a datatype which should not have a scale field"),
        };

        scale + add
    }

    pub fn has_table_name(&self) -> bool {
        match self {
            DataType::Text => true,
            DataType::NText => true,
            DataType::Image => true,
            _ => false,
        }
    }
}
