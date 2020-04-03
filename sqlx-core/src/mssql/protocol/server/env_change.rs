use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::Decode;
use crate::Error;
use byteorder::{LittleEndian, ReadBytesExt};
use core::str::from_utf8;

#[derive(Debug)]
pub enum EnvChange<'a> {
    Database(String),
    Language(String),
    CharacterSet(String),
    PacketSize(String),
    UnicodeDataSortingLocalId(String),
    UnicodeDataSortingComparisonFlags(String),
    SqlCollation(&'a [u8]),

    // TDS 7.2+
    BeginTransaction,
    CommitTransaction,
    RollbackTransaction,
    EnlistDtcTransaction,
    DefectTransaction,
    RealTimeLogShipping,
    PromoteTransaction,
    TransactionManagerAddress,
    TransactionEnded,
    ResetConnectionCompletionAck,
    LoginRequestUserNameAck,

    // TDS 7.4+
    RoutingInformation,
}

impl<'de> Decode<'de> for EnvChange<'de> {
    fn decode(mut buf: &'de [u8]) -> crate::Result<Self> {
        Ok(match buf.get_u8()? {
            1 => EnvChange::Database(buf.get_utf16_b_str()?),
            2 => EnvChange::Language(buf.get_utf16_b_str()?),
            3 => EnvChange::CharacterSet(buf.get_utf16_b_str()?),
            4 => EnvChange::PacketSize(buf.get_utf16_b_str()?),
            5 => EnvChange::UnicodeDataSortingLocalId(buf.get_utf16_b_str()?),
            6 => EnvChange::UnicodeDataSortingComparisonFlags(buf.get_utf16_b_str()?),
            7 => EnvChange::SqlCollation(buf.get_b_bytes()?),

            ty => {
                return Err(
                    protocol_err!("unexpected value {} for ENVCHANGE/TokenType", ty).into(),
                );
            }
        })
    }
}
