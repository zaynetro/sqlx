use crate::Result;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
// use std::borrow::Cow;
use crate::io::Buf;

pub trait DecodeBinXml<'a>: Sized {
    fn decode_xml(buf: &'a [u8]) -> Result<Self>;
}

pub enum Token {
    SqlSmallInt = 0x01,
    SqlInt = 0x02,
    SqlReal = 0x03,
    SqlFloat = 0x04,
    SqlMoney = 0x05,
    SqlBit = 0x06,
    SqlTinyInt = 0x07,
    SqlBigInt = 0x08,
    SqlUuid = 0x09,
    SqlDecimal = 0x0A,
    SqlNumeric = 0x0B,
    SqlBinary = 0x0C,
    SqlChar = 0x0D,
    SqlNChar = 0x0E,
    SqlVarBinary = 0x0F,
    SqlVarChar = 0x10,
    SqlNVarChar = 0x11,
    SqlDateTime = 0x12,
    SqlSmallDateTime = 0x13,
    SqlSmallMoney = 0x14,
    SqlText = 0x16,
    SqlImage = 0x17,
    SqlNText = 0x18,
    SqlUdt = 0x1B,
    XsdTimeOffset = 0x7A,
    XsdDateTimeOffset = 0x7B,
    XsdDateOffset = 0x7C,
    XsdTime2 = 0x7D,
    XsdDateTime2 = 0x7E,
    XsdDate2 = 0x7F,
    XsdTime = 0x81,
    XsdDateTime = 0x82,
    XsdDate = 0x83,
    XsdBinHex = 0x84,
    XsdBase64 = 0x85,
    XsdBoolean = 0x86,
    XsdDecimal = 0x87,
    XsdByte = 0x88,
    XsdUnsignedShort = 0x89,
    XsdUnsignedInt = 0x8A,
    XsdUnsignedLong = 0x8B,
    XsdQName = 0x8C,
    FlushDefinedNameTokens = 0xE9,
    ExtenToken = 0xEA,
    EndnestToken = 0xEB,
    NestToken = 0xEC,
    QNameDefToken = 0xEF,
    NameDefToken = 0xF0,
    CDataEndToken = 0xF1,
    CDataToken = 0xF2,
    CommentToken = 0xF3,
    PiToken = 0xF4,
    EndAttributesToken = 0xF5,
    AttributeToken = 0xF6,
    EndElementToken = 0xF7,
    ElementToken = 0xF8,
    SubSetToken = 0xF9,
    PublicToken = 0xFA,
    SystemToken = 0xFB,
    DoctypeDeclToken = 0xFC,
    EncodingToken = 0xFD,
    XmlDeclToken = 0xFE,
}

pub type Signature = u16;
const SIGNATURE: Signature = 0xDFFF;

pub enum Version {
    Version1 = 0x01,
    Version2 = 0x02,
}

pub type Encoding = u16;

// 1200 little-endian = UTF-16LE
const ENCODING: Encoding = 0xB004;

pub enum StandAlone {
    NotSpecified = 0x00,
    Yes = 0x01,
    No = 0x02,
}

impl<'a> DecodeBinXml<'a> for StandAlone {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        match buf[0] {
            0x00 => Ok(StandAlone::NotSpecified),
            0x01 => Ok(StandAlone::Yes),
            0x02 => Ok(StandAlone::No),
            value => Err(protocol_err!(
                "Unprocessable standalone number. Expected 0, 1, or 2, but received: {:?}",
                value
            )
            .into()),
        }
    }
}

impl<'a> DecodeBinXml<'a> for Version {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        match buf[0] {
            0x01 => Ok(Version::Version1),
            0x02 => Ok(Version::Version2),
            value => Err(protocol_err!(
                "Unprocessable version number. Expected 1, or 2, but received: {:?}",
                value
            )
            .into()),
        }
    }
}

pub struct Document {
    signature: Signature,
    version: Version,
    encoding: Encoding,
}

impl<'a> DecodeBinXml<'a> for Document {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let signature = BigEndian::read_u16(&buf[0..]);
        let version = Version::decode_xml(&buf[2..])?;
        let encoding = BigEndian::read_u16(&buf[3..]);
        buf.advance(5);

        Ok(Self {
            signature,
            version,
            encoding,
        })
    }
}

// For converting `&[u8]` to `&[u16]` is unsafe and I don't want to deal
// with that right now.
// pub struct TextData<'a>(Cow<'a, str>);
pub struct TextData<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for TextData<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}

pub struct TextData64<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for TextData64<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb64::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}

pub struct XmlDecl<'a> {
    textdata1: TextData<'a>,
    textdata2: Option<TextData<'a>>,
}

impl<'a> DecodeBinXml<'a> for XmlDecl<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != Token::XmlDeclToken as u8 {
            return Err(protocol_err!("Expected XMLDECL_TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let textdata1 = TextData::<'a>::decode_xml(&mut &buf[0..])?;

        let textdata2 = if buf[0] == Token::XmlDeclToken as u8 {
            buf.advance(1);
            Some(TextData::<'a>::decode_xml(&mut &buf[0..])?)
        } else {
            None
        };

        StandAlone::decode_xml(&buf)?;
        buf.advance(1);

        Ok(Self {
            textdata1,
            textdata2,
        })
    }
}

pub enum Misc<'a> {
    Comment(Comment<'a>),
    Pi(Pi<'a>),
    // Metadata(Metadata),
}

pub struct Comment<'a>(TextData<'a>);

impl<'a> DecodeBinXml<'a> for Comment<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != Token::CommentToken as u8 {
            return Err(protocol_err!("Expected COMMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let text = TextData::<'a>::decode_xml(&mut &buf)?;

        Ok(Self(text))
    }
}

pub struct Pi<'a> {
    name: u32,
    textdata: TextData<'a>,
}

impl<'a> DecodeBinXml<'a> for Pi<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != Token::PiToken as u8 {
            return Err(protocol_err!("Expected PI-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let name = BigEndian::read_u32(&buf);
        buf.advance(4);

        let textdata = TextData::<'a>::decode_xml(&mut &buf)?;

        Ok(Self { name, textdata })
    }
}

pub enum Metadata<'a> {
    NameDef(NameDef<'a>),
    QNameDef(QNameDef),
    Extension(Extension<'a>),
    None,
}

impl<'a> DecodeBinXml<'a> for Metadata<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        // TODO: Clean this up. This is UUUGGGLLLLY!!
        let namedef = Token::NameDefToken as u8;
        let qnamedef = Token::QNameDefToken as u8;
        let flush = Token::FlushDefinedNameTokens as u8;
        let extension = Token::ExtenToken as u8;

        match buf[0] {
            namedef => {
                buf.advance(1);
                let name = NameDef::<'a>::decode_xml(&mut &buf)?;
                Ok(Metadata::NameDef(name))
            }

            qnamedef => {
                buf.advance(1);
                let name = QNameDef::decode_xml(&mut &buf)?;
                Ok(Metadata::QNameDef(name))
            }

            extension => {
                buf.advance(1);
                let extension = Extension::<'a>::decode_xml(&mut &buf)?;
                Ok(Metadata::Extension(extension))
            }

            flush => {
                buf.advance(1);
                Ok(Metadata::None)
            }

            value => {
                return Err(protocol_err!(
                    "Expected namedef, qnamedef, extension, or flush tokens, got: {:?}",
                    buf[0]
                )
                .into());
            }
        }
    }
}

pub struct Extension<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for Extension<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;
        let data = &buf[4..4 + length as usize];
        buf.advance(length as usize);
        Ok(Self(&data))
    }
}

pub struct NameDef<'a>(TextData<'a>);

impl<'a> DecodeBinXml<'a> for NameDef<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let text = TextData::<'a>::decode_xml(&mut &buf)?;
        Ok(Self(text))
    }
}

pub struct QNameDef {
    namespaceuri: Mb32,
    prefix: Mb32,
    localname: Mb32,
}

impl<'a> DecodeBinXml<'a> for QNameDef {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let namespaceuri = Mb32::decode_xml(&mut &buf)?;
        let prefix = Mb32::decode_xml(&mut &buf)?;
        let localname = Mb32::decode_xml(&mut &buf)?;

        Ok(Self {
            namespaceuri,
            prefix,
            localname,
        })
    }
}

pub struct Mb32(pub u32);

impl<'a> DecodeBinXml<'a> for Mb32 {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let mut value = 0u32;
        for _ in 0..4 {
            if buf[0] < 0x80 {
                break;
            }

            value = value << 8 + buf[0] as u32;
            buf.advance(1);
        }

        if buf[0] > 0x7F {
            return Err(protocol_err!("lowbyte is not in valid rage").into());
        }

        value = value << 8 & buf[0] as u32;
        buf.advance(1);

        Ok(Self(value))
    }
}

pub struct Mb64(pub u64);

impl<'a> DecodeBinXml<'a> for Mb64 {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let mut value = 0u64;
        for _ in 0..9 {
            if buf[0] < 0x80 {
                break;
            }

            value = value << 8 + buf[0] as u64;
            buf.advance(1);
        }

        if buf[0] > 0x7F {
            return Err(protocol_err!("lowbyte is not in valid rage").into());
        }

        value = value << 8 & buf[0] as u64;
        buf.advance(1);

        Ok(Self(value))
    }
}

pub struct Element {
    qname: Mb32,
    // attrs: Vec<Attribute<'a>>,
    // content: Content<'a>,
}

impl<'a> DecodeBinXml<'a> for Element {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        if buf[0] != Token::ElementToken as u8 {
            return Err(protocol_err!("Expected ELEMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let qname = Mb32::decode_xml(&mut &buf)?;

        // let mut attrs = Vec::new();
        // loop {
        //     attrs.push(Attribute::<'a>::decode_xml(&mut &buf)?);
        //     if buf[0] == Token::EndAttributesToken as u8 {
        //         break;
        //     }
        // }

        // let content = Content::<'a>::decode_xml(&mut &buf)?;

        if buf[0] != Token::EndElementToken as u8 {
            return Err(protocol_err!("Expected ENDELEMENT-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        Ok(Self {
            qname,
            // attrs,
            // content,
        })
    }
}

pub struct Attribute<'a> {
    metadata: Vec<Metadata<'a>>,
    qname: Mb32,
    // meta_or_value: Vec<MetaOrValue<'a>>
}

pub enum MetaOrValue<'a> {
    Meta(Metadata<'a>),
    // Value(AtomicValue<'a>),
}

impl<'a> DecodeBinXml<'a> for Attribute<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let mut metadata = Vec::new();
        loop {
            let mut temp_buf = &buf;
            let meta = Metadata::<'a>::decode_xml(&mut &temp_buf);
            match meta {
                Ok(meta) => {
                    metadata.push(meta);
                    buf = temp_buf;
                }

                Err(_) => {}
            }
        }

        if buf[0] != Token::AttributeToken as u8 {
            return Err(protocol_err!("Expected ATTRIBUTE-TOKEN, got: {:?}", buf[0]).into());
        }
        buf.advance(1);

        let qname = Mb32::decode_xml(&mut &buf)?;

        let mut meta_or_value: Vec<MetaOrValue> = Vec::new();

        loop {
            let mut temp_buf = &buf;

            let meta = Metadata::<'a>::decode_xml(&mut &temp_buf);
            match meta {
                Ok(meta) => {
                    meta_or_value.push(MetaOrValue::Meta(meta));
                    buf = temp_buf;
                    continue;
                }

                Err(_) => {}
            }

            // let temp_buf = &buf;
            // let value = AtomicValue::<'a>::decode_xml(&mut &temp_buf);
            // match value {
            //     Ok(value) => {
            //         metadata.push(MetaOrValue::Value(value));
            //         buf = temp_buf;
            //         continue;
            //     }

            //     Err(_) => {}
            // }
        }
    }
}

pub enum Sign {
    Positive = 0x01,
    Negative = 0x00,
}

impl<'a> DecodeBinXml<'a> for Sign {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        match buf[0] {
            0 => {
                buf.advance(1);
                Ok(Self::Negative)
            }

            1 => {
                buf.advance(1);
                Ok(Self::Positive)
            }

            _ => Err(protocol_err!("Sign is neither 0 or 1").into()),
        }
    }
}

pub enum AtomicValue<'a> {
    SqlSmallInt(i16),
    SqlInt(i32),
    SqlReal(f32),
    SqlFloat(f64),
    SqlMoney(u64),
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
    SqlSmallMoney(u32),
    SqlText(CodepageText64<'a>),
    SqlImage(Blob64<'a>),
    SqlNText(TextData<'a>),
    SqlUdt(Blob<'a>),
    // XsdTimeOffset(SqlDateTimeOffset),
    // XsdDateTimeOffset(SqlDateTimeOffset),
    // XsdDateOffset(SqlDateTimeOffset),
    // XsdTime2(SqlDateTime2),
    // XsdDateTime2(SqlDateTime2),
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

pub struct Blob<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for Blob<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}

pub struct Blob64<'a>(&'a [u8]);

impl<'a> DecodeBinXml<'a> for Blob64<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb64::decode_xml(&mut &buf)?.0;
        let data = &buf[4..length as usize];
        buf.advance(length as usize);
        Ok(Self(data))
    }
}

pub struct Decimal<'a> {
    byte: u8,
    sign: Sign,
    buf: &'a [u8],
}

impl<'a> DecodeBinXml<'a> for Decimal<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?;

        let byte = buf[0];
        buf.advance(1);

        let sign = Sign::decode_xml(&mut &buf)?;

        Ok(Self {
            byte,
            sign,
            buf: &buf[..length.0 as usize],
        })
    }
}

pub struct CodepageText<'a> {
    codepage: i32,
    buf: &'a [u8],
}

impl<'a> DecodeBinXml<'a> for CodepageText<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb32::decode_xml(&mut &buf)?.0;

        let codepage = BigEndian::read_i32(&buf);
        buf.advance(4);

        let data = &buf[4..length as usize];
        buf.advance(length as usize);

        Ok(Self {
            codepage,
            buf: data,
        })
    }
}

pub struct CodepageText64<'a> {
    codepage: i32,
    buf: &'a [u8],
}

impl<'a> DecodeBinXml<'a> for CodepageText64<'a> {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let length = Mb64::decode_xml(&mut &buf)?.0;

        let codepage = BigEndian::read_i32(&buf);
        buf.advance(4);

        let data = &buf[4..length as usize];
        buf.advance(length as usize);

        Ok(Self {
            codepage,
            buf: data,
        })
    }
}

pub struct SqlDate(u32);

impl<'a> DecodeBinXml<'a> for SqlDate {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let date = ((buf[0] as u32) << 16) & (buf[1] as u32) & ((buf[0] as u32) << 8);
        Ok(Self(date))
    }
}
