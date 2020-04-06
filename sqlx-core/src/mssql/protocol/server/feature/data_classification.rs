use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Whenever a login response stream is sent for a TDS connection whose login request includes a
// DATACLASSIFICATION FeatureExt token, the server login response message stream SHOULD<54>be
// capable of optionally containing a FEATUREEXTACK token by including the DATACLASSIFICATION
// FeatureId in the FEATUREEXTACK token stream. The corresponding FeatureAckData MUST then
// include the following information that indicates whether the server supports data
// classification and to what extent. The FeatureAckData format is as follows:
//
//      DATACLASSIFICATION_VERSION = BYTE
//      IsEnabled                  = BYTE
//      VersionSpecificData = *2147483647BYTE ; The actual length of data is FeatureAckDataLen - 2
//
//      FeatureAckData           = DATACLASSIFICATION_VERSION
//                                 IsEnabled
//                                 VersionSpecificData
//
// DATACLASSIFICATION_VERSION: This field specifies the version number of the data classification
// information that is to be used for this connection. This value MUST be 1 or 2, as specified for
// DATACLASSIFICATION_VERSION in section 2.2.6.4.
//
// IsEnabled: This field specifies whether the server supports data classification.
//The values of this field are as follows:
//
//      0 = The server does not support data classification.
//      1 = The server supports data classification.
//
// VersionSpecificData: This field specifies which version of data classification information
// is returned. The values of this field are as follows:
//
//      When the value of the DATACLASSIFICATION_VERSION field is 1, the response in the feature
//      extension acknowledgement contains no version-specific data.
#[derive(Debug)]
pub struct FeatureDataClassification<'a> {
    version: u8,
    is_enabled: bool,
    data: Option<&'a [u8]>,
}

impl<'a> Decode<'a> for FeatureDataClassification<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let version = buf.get_u8()?;
        let is_enabled = buf.get_u8()? == 1;
        let data = if version == 1 { None } else { Some(&buf[..]) };

        Ok(Self {
            version,
            is_enabled,
            data,
        })
    }
}
