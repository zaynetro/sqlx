use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

pub mod azure_sql_support;
pub mod column_encryption;
pub mod data_classification;
pub mod fed_auth;
pub mod global_transactions;
pub mod session_recovery;
pub mod utf8_support;

pub use azure_sql_support::FeatureAzureSqlSupport;
pub use column_encryption::FeatureColumnEncryption;
pub use data_classification::FeatureDataClassification;
pub use fed_auth::FeatureFedAuth;
pub use global_transactions::FeatureGlobalTransactions;
pub use session_recovery::SessionState;
pub use utf8_support::FeatureUtf8Support;

#[derive(Debug)]
pub enum Feature<'a> {
    SessionRecovery(SessionState<'a>),
    FedAuth(FeatureFedAuth),
    ColumnEncryption(FeatureColumnEncryption<'a>),
    GlobalTransactions(FeatureGlobalTransactions),
    AzureSqlSupport(FeatureAzureSqlSupport),
    DataClassification(FeatureDataClassification<'a>),
    Utf8Support(FeatureUtf8Support),
}

#[derive(Debug)]
pub enum FeatureId {
    SessionRecovery = 0x01,
    FedAuth = 0x02,
    ColumnEncryption = 0x04,
    GlobalTransactions = 0x05,
    AzureSqlSupport = 0x08,
    DataClassification = 0x09,
    Utf8Support = 0x0A,
}

impl From<u8> for FeatureId {
    fn from(value: u8) -> Self {
        match value {
            0x01 => FeatureId::SessionRecovery,
            0x02 => FeatureId::FedAuth,
            0x04 => FeatureId::ColumnEncryption,
            0x05 => FeatureId::GlobalTransactions,
            0x08 => FeatureId::AzureSqlSupport,
            0x09 => FeatureId::DataClassification,
            0x0A => FeatureId::Utf8Support,
            v => panic!("Unrecognized FeatureId {:?}", v),
        }
    }
}

impl<'a> Decode<'a> for Feature<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let id = FeatureId::from(buf.get_u8()?);
        let len = buf.get_u32::<LittleEndian>()? as usize;

        Ok(match id {
            FeatureId::SessionRecovery => {
                Feature::SessionRecovery(SessionState::decode(&buf[..len])?)
            }
            FeatureId::FedAuth => {
                // TODO: Handle Security Token
                Feature::FedAuth(FeatureFedAuth::decode_live_id_compact(&buf[..len])?)
            }
            FeatureId::ColumnEncryption => {
                Feature::ColumnEncryption(FeatureColumnEncryption::decode(&buf[..len])?)
            }
            FeatureId::GlobalTransactions => {
                Feature::GlobalTransactions(FeatureGlobalTransactions::decode(&buf[..len])?)
            }
            FeatureId::AzureSqlSupport => {
                Feature::AzureSqlSupport(FeatureAzureSqlSupport::decode(&buf[..len])?)
            }
            FeatureId::DataClassification => {
                Feature::DataClassification(FeatureDataClassification::decode(&buf[..len])?)
            }
            FeatureId::Utf8Support => {
                Feature::Utf8Support(FeatureUtf8Support::decode(&buf[..len])?)
            }
        })
    }
}
