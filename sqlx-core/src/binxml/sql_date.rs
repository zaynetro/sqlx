use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;
use byteorder::{BigEndian, ByteOrder};

/// number of days since 0001-1-1
pub struct SqlDate(u32);

impl<'a> DecodeBinXml<'a> for SqlDate {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let date = ((buf[2] as u32) << 16) & (buf[0] as u32) & ((buf[1] as u32) << 8);
        Ok(Self(date))
    }
}

pub struct SqlTimeZone(i16);

impl<'a> DecodeBinXml<'a> for SqlTimeZone {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let timezone = BigEndian::read_i16(&buf);
        buf.advance(2);

        Ok(Self(timezone))
    }
}

pub struct SqlTime(u64);

impl<'a> DecodeBinXml<'a> for SqlTime {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let percision = buf.next();

        let value: Result<u64> = match percision {
            0..=2 => {
                let value = BigEndian::read_u16(&buf);
                let value = value << 8 + buf[2] as u64;
                buf.advance(3);
                Ok(value as u64)
            }

            3..=4 => {
                let value = BigEndian::read_u32(&buf);
                buf.advance(4);
                Ok(value as u64)
            }

            5..=7 => {
                let value = BigEndian::read_u32(&buf) as u64;
                let value = value << 8 + buf[4] as u64;
                buf.advance(5);
                Ok(value as u64)
            }

            value => Err(protocol_err!(
                "SqlTime Percision is restricted to 0-7, but value is: {:?}",
                value
            )
            .into()),
        };

        // 00:00:00.0000000 through 23:59:59.9999999
        // Convert value into nanoseconds since beginning of the day
        // Percision indicates number of 1/10 seconds since the beginning of the day
        // This means max percision 7 is 100ns increments. Converting 100ns into nanos
        // simply multiply by 100 or 10 ^ (9 - 7)
        Ok(Self(value? * 10u64.pow(9 - percision as u32)))
    }
}

pub struct SqlDateTime2 {
    time: SqlTime,
    date: SqlDate,
}

impl<'a> DecodeBinXml<'a> for SqlDateTime2 {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let time = SqlTime::decode_xml(&mut &buf)?;
        let date = SqlDate::decode_xml(&mut &buf)?;

        Ok(Self { time, date })
    }
}

pub struct SqlDateTimeOffset {
    time: SqlTime,
    date: SqlDate,
    timezone: SqlTimeZone,
}

impl<'a> DecodeBinXml<'a> for SqlDateTimeOffset {
    fn decode_xml(buf: &'a [u8]) -> Result<Self> {
        let time = SqlTime::decode_xml(&mut &buf)?;
        let date = SqlDate::decode_xml(&mut &buf)?;
        let timezone = SqlTimeZone::decode_xml(&mut &buf)?;

        Ok(Self {
            time,
            date,
            timezone,
        })
    }
}
