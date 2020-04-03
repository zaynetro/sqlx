use crate::io::Buf;
use crate::mssql::protocol::Decode;
use crate::Error;

#[derive(Debug)]
pub enum EnvChange {
    Database(String),
    Language(String),
    CharacterSet(String),
    PacketSize(String),
    UnicodeDataSortingLocalId(String),
    UnicodeDataSortingComparisonFlags(String),
    SqlCollation(Vec<u8>),

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

impl<'de> Decode<'de> for EnvChange {
    fn decode(mut buf: &'de [u8]) -> crate::Result<Self> {
        match buf.get_u8()? {
            ty => Err(protocol_err!("unexpected value {} for ENVCHANGE/TokenType", ty).into()),
        }
    }
}
