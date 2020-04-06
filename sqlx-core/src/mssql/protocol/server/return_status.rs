use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Used to send the status value of an RPCto the client. The server also uses this token to
//      send the result status value of a T-SQL EXEC query.
//
// Token Stream Comments:
//      - The token value is 0x79.
//      - This token MUST be returned to the client when an RPC is executed by the server.
//
// Token Stream Definition:
//      RETURNSTATUS  = TokenType
//                      Value
#[derive(Debug)]
pub struct ReturnStatus {
    // The return status value determined by the remote procedure. Return status MUST NOT be NULL.
    value: i32,
}

impl Decode<'_> for ReturnStatus {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let value = buf.get_i32::<LittleEndian>()?;

        Ok(Self { value })
    }
}
