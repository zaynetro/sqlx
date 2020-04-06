use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Introduced in TDS 7.4, FEATUREEXTACK is used to send an optional acknowledge message to
//      the client for features that are defined in FeatureExt. The token stream is sent only
//      along with the LOGINACK in a Login Response message.
//
// Token Stream Comments:
//      The token value is 0xAE.
//
// Token Stream Definition:
//      FEATUREEXTACK =
//          TokenType
//          1*FeatureAckOpt
#[derive(Debug)]
pub struct FeatureExtAck<'a> {
    options: Vec<FeatureExtOpt<'a>>,
}

#[derive(Debug)]
pub struct FeatureExtOpt<'a> {
    // The unique identifier number of a feature. Each feature MUST use the same ID number here as
    // in FeatureExt. If the client did not send a request for a specific feature but the FeatureId
    // is returned, the client MUST consider it as a TDS Protocol error and MUST terminate the
    // connection. Each feature defines its own logic if it wants to use FeatureAckOpt to send
    // information back to the client during the login response. The features available to use by a
    // FeatureId are defined in the following table.
    id: u8,
    // The length of FeatureAckData, in bytes.
    data_len: u32,
    // The acknowledge data of a specific feature. Each feature SHOULD define its own data format
    // in the FEATUREEXTACK token if it is selected to acknowledge the feature.
    data: &'a [u8],
}

impl<'a> Decode<'a> for FeatureExtOpt<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let id = buf.get_u8()?;
        let data_len = buf.get_u32::<LittleEndian>()?;
        let data = &buf[0..data_len as usize];

        Ok(Self { id, data_len, data })
    }
}

impl<'a> Decode<'a> for FeatureExtAck<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let mut options = vec![FeatureExtOpt::decode(buf)?];

        while buf[0] != 0xFF {
            options.push(FeatureExtOpt::decode(buf)?);
        }

        Ok(Self { options })
    }
}
