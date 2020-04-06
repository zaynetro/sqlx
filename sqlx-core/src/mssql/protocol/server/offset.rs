use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Used to inform the client where in the client's SQL text buffer a particular keyword occurs.
//
// Token Stream Comments:
//      - The token value is 0x78.
//      - The token was removed in TDS 7.2.
//
// Token Stream Definition:
//      OFFSET          = TokenType          ; (removed in TDS 7.2)
//                        Identifier
//                        OffSetLen
#[derive(Debug)]
pub struct Offset {
    // The keyword to which OffSetLen refers.
    id: u16,
    // The offset in the SQL text buffer received by the server of the identifier. The SQL text
    // buffer begins with an OffSetLen value of 0 (MOD 64 kilobytes if value of OffSet
    // is larger than 64 kilobytes).
    offset: u16,
}

impl Decode<'_> for Offset {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let id = buf.get_u16::<LittleEndian>()?;
        let offset = buf.get_u16::<LittleEndian>()?;

        Ok(Self { id, offset })
    }
}
