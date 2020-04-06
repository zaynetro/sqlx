use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Whenever a login response stream is sent for a TDS connection whose login request includes a
// GLOBALTRANSACTIONS FeatureExt token, the server login response message stream can optionally
// include a FEATUREEXTACK token by including the GLOBALTRANSACTIONS FeatureId in the
// FEATUREEXTACK token stream. The corresponding FeatureAckData MUST then include a flag that
// indicates whether the server supports Global Transactions.
// The FeatureAckData format is as follows:
//
//      IsEnabled           = BYTE
//      FeatureAckData      = IsEnabled
//
// IsEnabled: Specifies whether the server supports Global Transactions.
// The values of this field are as follows:
//
//      0 = The server does not support Global Transactions.
//      1 = The server supports Global Transactions.
#[derive(Debug)]
pub struct FeatureGlobalTransactions {
    is_enabled: bool,
}

impl Decode<'_> for FeatureGlobalTransactions {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        Ok(Self {
            is_enabled: buf.get_u8()? == 1,
        })
    }
}
