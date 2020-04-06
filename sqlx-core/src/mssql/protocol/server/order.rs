use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Used to inform the client by which columns the data is ordered
//
// Token Stream Comments:
//      - The token value is 0xA9.
//      - This token is sent only in the event that an ORDER BY clause is executed.
//
// Token Stream Definition:
//      OFFSET          = TokenType          ; (removed in TDS 7.2)
//                        Length
//                        ColNum
#[derive(Debug)]
pub struct Order {
    // The total length of the ORDER data stream
    length: u16,
    // The column number in the result set.
    col_nums: Vec<u16>,
}

impl Decode<'_> for Order {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let length = buf.get_u16::<LittleEndian>()?;

        let mut col_nums = Vec::new();
        for _ in (0..length as usize / 2) {
            col_nums.push(buf.get_u16::<LittleEndian>()?);
        }

        Ok(Self { length, col_nums })
    }
}
