use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Used to send session state data to the client. The data format defined here can also be
//      used to send session state data for session recovery during login and login response.
//
// Token Stream Comments:
//      - The token value is 0xE4.
//      - This token stream MUST NOT be sent if the SESSIONRECOVERY feature is not negotiated
//        on the connection.
//      - When this token stream is sent, the next token MUST be DONEor DONEPROCwith DONE_FINAL.
//      - If the SESSIONRECOVERY feature is negotiated on the connection, the server SHOULD send
//        this token to the client to inform any session state update.
//
// Token Stream Definition:
//      SESSIONSTATE =
//          TokenType
//          Length
//          SeqNo
//          Status
//          SessionStateDataSet
#[derive(Debug)]
pub struct SessionState<'a> {
    // The length, in bytes, of the token stream (excluding TokenType and Length).
    length: u32,
    // The sequence number of the SESSIONSTATE token in the connection. This number, which starts
    // at 0 and increases by one each time, can be used to track the order of SESSIONSTATE tokens
    // sent during the course of a connection. The SeqNo applies to all StateIds in the token. If
    // the SeqNo for any StateId reaches %xFFFFFFFF, both client and server MUST consider that
    // the SESSIONRECOVERY feature is permanently disabled on the connection. The server SHOULD
    // send a token with fRecoverable set to FALSE to disable SESSIONRECOVERY for this session.
    // The client SHOULD NOT set either ResetConn bit (RESETCONNECTION or RESETCONNECTIONSKIPTRAN)
    // on the connection once it receives any SeqNo of %xFFFFFFFF because ResetConn could reset a
    // connection back to an initial recoverable state and SESSIONRECOVERY needs to be permanently
    // disabled on the connection in this case. If the server does receive ResetConn after SeqNo
    // reaches %xFFFFFFFF, it SHOULD reuse this same SeqNo to disable SESSIONRECOVERY.The client
    // SHOULD track SeqNo for each StateId and keep the latest data for session recovery.
    seq_no: u32,
    // Status of the session StateId in this token.
    status: Status,
    data_set: Vec<SessionData<'a>>,
}

bitflags! {
    pub struct Status: u8 {
        // fRecoverable: TRUE means all session StateIds in this token are recoverable.The client
        // SHOULD track Status for each StateId and keep the latest data for session recovery.
        // A client MUST NOT try to recover a dead connection unless fRecoverable is TRUE for
        // all session StateIds received from server.
        const RECOVERABLE = 0x01;
    }
}

#[derive(Debug)]
pub struct SessionData<'a> {
    // The identification number of the session state. %xFF is reserved.
    id: u8,
    // The length, in bytes, of the corresponding StateValue. If the length is 254 bytes or
    // smaller, one BYTE is used to represent the field. If the length is 255 bytes or
    // larger, %xFF followed by a DWORD is used to represent the field. If this field is 0,
    // client SHOULD skip sending SessionStateData for the StateId during session recovery.
    len: u32,
    // The value of the session state. This can be any arbitrary data as long as the
    // server understands it.
    value: &'a [u8],
}

impl<'a> Decode<'a> for SessionState<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let length = buf.get_u32::<LittleEndian>()?;
        let seq_no = buf.get_u32::<LittleEndian>()?;
        let status = Status::from_bits_truncate(buf.get_u8()?);

        let mut data_set = Vec::new();
        loop {
            match SessionData::decode(buf) {
                Ok(v) => data_set.push(v),
                Err(_) => break,
            }
        }

        Ok(Self {
            length,
            seq_no,
            status,
            data_set,
        })
    }
}

impl<'a> Decode<'a> for SessionData<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let id = buf.get_u8()?;

        let mut len = buf.get_u8()? as u32;
        if len == 0xFF {
            len = buf.get_u32::<LittleEndian>()?;
        };

        let value = &buf[0..len as usize];

        Ok(Self { id, len, value })
    }
}
