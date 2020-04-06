use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// The presence of the COLUMNENCRYPTION FeatureExt SHOULD indicate that the client is capable of
// performing cryptographic operations on data. The feature data is described as follows:
//      Length                        = BYTE
//      COLUMNENCRYPTION_VERSION      = BYTE
//      FeatureData                   = COLUMNENCRYPTION_VERSION
//                                      [Length EnclaveType]
//
// COLUMNENCRYPTION_VERSION: This field defines the cryptographic protocol version that the client
// understands. The values of this field are as follows:
//      1 = The client supports column encryption without enclave computations.
//      2 = The client SHOULD support column encryption when encrypted data require
//          enclave computations.
//
// EnclaveType: This field is a string that SHOULD be populated by the server and used by the
// client to identify the type of enclave that the server is configured to use. During login for
// the initial connection, the client can request COLUMNENCRYPTION with Length as 1 and
// COLUMNENCRYPTION_VERSION as either 1 or 2. When the client requests COLUMNENCRYPTION_VERSION
// as 2, the server MUST return COLUMNENCRYPTION_VERSION as 2 together with the value of
// EnclaveType, if the server contains an enclave that is configured for use. If EnclaveType is
// not returned and the column encryption version is returned as 2, the client driver
// MUST raise an error.
#[derive(Debug)]
pub struct FeatureColumnEncryption<'a> {
    version: u8,
    enclave_type: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for FeatureColumnEncryption<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let version = buf.get_u8()?;
        let enclave_type = if version == 0x02 {
            let length = buf.get_u8()?;
            Some(&buf[0..length as usize])
        } else {
            None
        };

        Ok(Self {
            version,
            enclave_type,
        })
    }
}
