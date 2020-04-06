use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// The presence of the AZURESQLSUPPORT FeatureExt indicates whether failover partner login with
// read-only intent to Azure SQL Database MAY be supported. For information about
// failover partner, see [MSDOCS-DBMirror].
//
// Whenever a login response stream is sent for a TDS connection whose login request includes an
// AZURESQLSUPPORT FeatureExt token, the server login response message stream can optionally
// include a FEATUREEXTACK token by setting the corresponding feature switch in Azure SQL Database.
// If it is included, the FEATUREEXTACK token stream MUST include the AZURESQLSUPPORT FeatureId.
//
//      FeatureAckData      = BYTE
//
// BYTE: The Bit 0 flag specifies whether failover partner login with read-only intent is supported.
// The values of this BYTE are as follows:
//      0 = The server does not support the AZURESQLSUPPORT feature extension.
//      1 = The server supports the AZURESQLSUPPORT feature extension.
#[derive(Debug)]
pub struct FeatureAzureSqlSupport {
    is_supported: bool,
}

impl Decode<'_> for FeatureAzureSqlSupport {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        Ok(Self {
            is_supported: buf.get_u8()? == 1,
        })
    }
}
