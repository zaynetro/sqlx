use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Whenever a login response stream is sent for a TDS connection whose login request includes a
// FEDAUTH FeatureExt, the server login response message stream MUST include a FEATUREEXTACK token,
// and the FEATUREEXTACK token stream MUST include the FEDAUTH FeatureId. The format is described
// below based on the bFedAuthLibrary that is used in FEDAUTH FeatureExt.
//
// When the bFedAuthLibrary is Live ID Compact Token, the format is as follows:
//      Nonce               = 32BYTE
//      Signature           = 32BYTE
//      FeatureAckData      = Nonce
//                            Signature
//
// Nonce: The client-specified nonce in PRELOGIN.
// Signature: The HMAC-SHA-256 [RFC6234]of the client-specified nonce, using the session key
//      retrieved from the federated authenticationcontext as the shared secret.
//
// When the bFedAuthLibrary is Security Token, the format is as follows:
//      Nonce               = 32BYTE
//      FeatureAckData      = [Nonce]
//
// Nonce: The client-specified nonce in PRELOGIN. This field MUST be present if the client’s
// PRELOGIN message included a NONCE field. Otherwise, this field MUST NOT be present.
#[derive(Debug)]
pub struct FeatureFedAuth {
    nonce: [u8; 32],
    signature: Option<[u8; 32]>,
}

impl FeatureFedAuth {
    pub fn decode_live_id_compact(mut buf: &[u8]) -> crate::Result<Self> {
        let mut nonce = [0u8; 32];
        let mut signature = [0u8; 32];

        nonce.copy_from_slice(&buf[0..32]);
        signature.copy_from_slice(&buf[32..64]);
        buf.advance(64);

        Ok(Self {
            nonce,
            signature: Some(signature),
        })
    }

    pub fn decode_security(mut buf: &[u8]) -> crate::Result<Self> {
        let mut nonce = [0u8; 32];

        nonce.copy_from_slice(&buf[0..32]);
        buf.advance(32);

        Ok(Self {
            nonce,
            signature: None,
        })
    }
}
