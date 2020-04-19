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

impl Default for DataType {
    fn default() -> Self {
        Self::Null
    }
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
        matches!(
            self,
            DataType::Null
                | DataType::Int1
                | DataType::Big
                | DataType::Int2
                | DataType::Int4
                | DataType::DateTime4
                | DataType::Float4
                | DataType::Money
                | DataType::DateTime
                | DataType::Float8
                | DataType::Money4
        )
    }

    pub fn is_bytelen(&self) -> bool {
        matches!(
            self,
            DataType::Guid
                | DataType::IntN
                | DataType::Decimal
                | DataType::Numeric
                | DataType::BitN
                | DataType::DecimalN
                | DataType::NumericN
                | DataType::FloatN
                | DataType::MoneyN
                | DataType::DateTimeN
                | DataType::DateN
                | DataType::TimeN
                | DataType::DateTime2N
                | DataType::DateTimeOffsetN
                | DataType::Char
                | DataType::VarChar
                | DataType::Binary
                | DataType::VarBinary
        )
    }

    pub fn is_ushort_len(&self) -> bool {
        matches!(
            self,
            DataType::BigVarBinary
                | DataType::BigVarChar
                | DataType::BigBinary
                | DataType::BigChar
                | DataType::NVarChar
                | DataType::NChar
        )
    }

    pub fn is_long_len(&self) -> bool {
        matches!(
            self,
            DataType::Xml
                | DataType::Text
                | DataType::Image
                | DataType::NText
                | DataType::SsVariant
        )
    }

    pub fn is_var_len(&self) -> bool {
        self.is_bytelen() || self.is_ushort_len() || self.is_long_len()
    }

    pub fn part_len(&self) -> bool {
        matches!(
            self,
            DataType::Xml
                | DataType::BigVarChar
                | DataType::BigVarBinary
                | DataType::NVarChar
                | DataType::Udt
        )
    }

    pub fn has_collation(&self) -> bool {
        matches!(
            self,
            DataType::BigChar
                | DataType::BigVarChar
                | DataType::Text
                | DataType::NText
                | DataType::NChar
                | DataType::NVarChar
        )
    }

    pub fn has_precision_and_scale(&self) -> bool {
        matches!(
            self,
            DataType::Numeric | DataType::NumericN | DataType::Decimal | DataType::DecimalN
        )
    }

    pub fn has_only_scale(&self) -> bool {
        matches!(
            self,
            DataType::TimeN | DataType::DateTime2N | DataType::DateTimeOffsetN
        )
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
        matches!(self, DataType::Text | DataType::NText | DataType::Image)
    }

    pub fn is_charbin_null(&self) -> bool {
        matches!(
            self,
            DataType::BigChar
                | DataType::BigVarChar
                | DataType::NChar
                | DataType::NVarChar
                | DataType::BigBinary
                | DataType::BigVarBinary
        )
    }

    pub fn fixed_len(&self) -> usize {
        todo!()
    }
}
