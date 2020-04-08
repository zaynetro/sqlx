use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Used to send an error message to the client.
//
// Token Stream Comments:
//      The token value is 0xAA.
//
// Token Stream Definition:
//      ERROR =
//          TokenType
//          Length
//          Number
//          State
//          Class
//          MsgText
//          ServerName
//          ProcName
//          LineNumber
#[derive(Debug)]
pub struct Error {
    // The error number
    number: i32,
    // The error state, used as a modifier to the error number.
    state: u8,
    // The class (severity) of the error. A class of less than 10 indicates
    // an informational message.
    class: u8,
    // The message text length and message text using US_VARCHAR format.
    msg_text: String,
    // The server name length and server name using B_VARCHAR format
    server_name: String,
    // The stored procedurename length and the stored procedure name using B_VARCHAR format
    proc_name: String,
    // The line number in the SQL batch or stored procedure that caused the error. Line numbers
    // begin at 1. If the line number is not applicable to the message, the
    // value of LineNumber is 0.
    line_number: i32,
}

impl Decode<'_> for Error {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let number = buf.get_i32::<LittleEndian>()?;
        let state = buf.get_u8()?;
        let class = buf.get_u8()?;
        let msg_text = buf.get_utf16_us_str()?;
        let server_name = buf.get_utf16_b_str()?;
        let proc_name = buf.get_utf16_b_str()?;
        let line_number = buf.get_i32::<LittleEndian>()?;

        Ok(Self {
            number,
            state,
            class,
            msg_text,
            server_name,
            proc_name,
            line_number,
        })
    }
}
