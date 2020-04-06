use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// The presence of the UTF8_SUPPORT FeatureExtAck token in the response message stream indicates
// whether the server’s ability to receive and send UTF-8 encoded data SHOULD be supported.
//
// Whenever a login response stream is sent for a TDS connection whose login request includes a
// UTF8_SUPPORT FeatureExt token, the server login response message stream can optionally include
// a FEATUREEXTACK token. If that token is included, the FEATUREEXTACK token MUST include the
// UTF8_SUPPORT FeatureId and the appropriate feature acknowledgement data. The FeatureAckData
// format is as follows:
//      FeatureAckData = BYTE
//
// BYTE: The Bit 0 value specifies whether the server can receive and send UTF-8 encoded data.
// The values of this BYTE are as follows:
//      0 = The server does not support the UFT8_SUPPORT feature extension
//      1 = The server supports the UTF8_SUPPORT feature extension.
#[derive(Debug)]
pub struct FeatureUtf8Support {
    is_supported: bool,
}

impl Decode<'_> for FeatureUtf8Support {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        Ok(Self {
            is_supported: buf.get_u8()? == 1,
        })
    }
}
