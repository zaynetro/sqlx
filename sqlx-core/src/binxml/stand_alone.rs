use super::DecodeBinXml;
use crate::io::Buf;
use crate::Result;

pub enum StandAlone {
    NotSpecified = 0x00,
    Yes = 0x01,
    No = 0x02,
}

impl<'a> DecodeBinXml<'a> for StandAlone {
    fn decode_xml(mut buf: &'a [u8]) -> Result<Self> {
        let result: Result<Self> = match buf[0] {
            0x00 => Ok(StandAlone::NotSpecified),
            0x01 => Ok(StandAlone::Yes),
            0x02 => Ok(StandAlone::No),
            value => Err(protocol_err!(
                "Unprocessable standalone number. Expected 0, 1, or 2, but received: {:?}",
                value
            )
            .into()),
        };

        buf.advance(1);

        Ok(result?)
    }
}
