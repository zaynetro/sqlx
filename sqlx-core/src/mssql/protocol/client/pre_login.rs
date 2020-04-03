use crate::io::{Buf, BufMut};
use crate::mssql::protocol::{Decode, Encode, PacketType};
use bitflags::bitflags;
use byteorder::BigEndian;
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

const TERMINATOR: u8 = 0xFF;

/// A message sent by the client to set up context for login. The server responds to a client
/// `PRELOGIN` message with a message of packet header type `0x04` and the packet data
/// containing a `PRELOGIN` structure.
#[derive(Debug, Default)]
pub struct PreLogin<'a> {
    pub version: Version,
    pub encryption: Encrypt,
    pub instance: Option<&'a str>,
    pub thread_id: Option<u32>,
    pub trace_id: Option<TraceId>,
    pub multiple_active_result_sets: Option<bool>,
}

impl<'de> Decode<'de> for PreLogin<'de> {
    fn decode(mut buf: &'de [u8]) -> crate::Result<Self> {
        let mut version = None;
        let mut encryption = None;

        // TODO: Decode the remainder of the structure
        // let mut instance = None;
        // let mut thread_id = None;
        // let mut trace_id = None;
        // let mut multiple_active_result_sets = None;

        let mut offsets = buf;

        loop {
            let token = offsets.get_u8()?;

            match PreLoginOptionToken::decode(token) {
                Some(token) => {
                    let offset = offsets.get_u16::<BigEndian>()? as usize;
                    let size = offsets.get_u16::<BigEndian>()? as usize;
                    let mut data = &buf[offset..offset + size];

                    match token {
                        PreLoginOptionToken::Version => {
                            let major = data.get_u8()?;
                            let minor = data.get_u8()?;
                            let build = data.get_u16::<BigEndian>()?;
                            let sub_build = data.get_u16::<BigEndian>()?;

                            version = Some(Version {
                                major,
                                minor,
                                build,
                                sub_build,
                            });
                        }

                        PreLoginOptionToken::Encryption => {
                            encryption = Some(Encrypt::from_bits_truncate(data.get_u8()?));
                        }

                        tok => todo!("{:?}", tok),
                    }
                }

                None if token == TERMINATOR => {
                    break;
                }

                None => {
                    return Err(protocol_err!(
                        "PRELOGIN: unexpected login option token: 0x{:02?}",
                        token
                    )
                    .into());
                }
            }
        }

        let version =
            version.ok_or(protocol_err!("PRELOGIN: missing required `version` option"))?;

        let encryption = encryption.ok_or(protocol_err!(
            "PRELOGIN: missing required `encryption` option"
        ))?;

        Ok(Self {
            version,
            encryption,
            ..Default::default()
        })
    }
}

impl Encode for PreLogin<'_> {
    #[inline]
    fn r#type() -> PacketType {
        PacketType::PreLogin
    }

    fn encode(&self, buf: &mut Vec<u8>) {
        use PreLoginOptionToken::*;

        // NOTE: Packet headers are written in MsSqlStream::write

        // Rules
        //  PRELOGIN = (*PRELOGIN_OPTION *PL_OPTION_DATA) / SSL_PAYLOAD
        //  PRELOGIN_OPTION = (PL_OPTION_TOKEN PL_OFFSET PL_OPTION_LENGTH) / TERMINATOR

        // Count the number of set options
        let num_options = 2
            + self.instance.map_or(0, |_| 1)
            + self.thread_id.map_or(0, |_| 1)
            + self.trace_id.as_ref().map_or(0, |_| 1)
            + self.multiple_active_result_sets.map_or(0, |_| 1);

        // Calculate the length of the option offset block. Each block is 5 bytes and it ends in
        // a 1 byte terminator.
        let len_offsets = (num_options * 5) + 1;
        let mut offsets = buf.len() as usize;
        let mut offset = len_offsets as u16;

        // Reserve a chunk for the offset block and set the final terminator
        buf.advance(len_offsets);
        let end_offsets = buf.len() - 1;
        buf[end_offsets] = TERMINATOR;

        // NOTE: VERSION is a required token, and it MUST be the first token.
        Version.encode(buf, &mut offsets, &mut offset, 6);
        self.version.encode(buf);

        Encryption.encode(buf, &mut offsets, &mut offset, 1);
        buf.push(self.encryption.bits());

        if let Some(name) = self.instance {
            Instance.encode(buf, &mut offsets, &mut offset, name.len() as u16 + 1);
            buf.extend_from_slice(name.as_bytes());
            buf.push(b'\0');
        }

        if let Some(id) = self.thread_id {
            ThreadId.encode(buf, &mut offsets, &mut offset, 4);
            buf.extend_from_slice(&id.to_le_bytes());
        }

        if let Some(trace) = &self.trace_id {
            ThreadId.encode(buf, &mut offsets, &mut offset, 36);
            buf.extend_from_slice(trace.connection_id.as_bytes());
            buf.extend_from_slice(trace.activity_id.as_bytes());
            buf.extend_from_slice(&trace.activity_seq.to_be_bytes());
        }

        if let Some(mars) = &self.multiple_active_result_sets {
            MultipleActiveResultSets.encode(buf, &mut offsets, &mut offset, 1);
            buf.push(*mars as u8);
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
enum PreLoginOptionToken {
    Version = 0x00,
    Encryption = 0x01,
    Instance = 0x02,
    ThreadId = 0x03,
    MultipleActiveResultSets = 0x04, // MARS
    TraceId = 0x05,
}

impl PreLoginOptionToken {
    fn encode(self, buf: &mut Vec<u8>, pos: &mut usize, offset: &mut u16, len: u16) {
        buf[*pos] = self as u8;
        *pos += 1;

        buf[*pos..(*pos + 2)].copy_from_slice(&offset.to_be_bytes());
        *pos += 2;

        buf[*pos..(*pos + 2)].copy_from_slice(&len.to_be_bytes());
        *pos += 2;

        *offset += len;
    }

    fn decode(b: u8) -> Option<Self> {
        Some(match b {
            0x00 => PreLoginOptionToken::Version,
            0x01 => PreLoginOptionToken::Encryption,
            0x02 => PreLoginOptionToken::Instance,
            0x03 => PreLoginOptionToken::ThreadId,
            0x04 => PreLoginOptionToken::MultipleActiveResultSets,
            0x05 => PreLoginOptionToken::TraceId,

            _ => {
                return None;
            }
        })
    }
}

bitflags! {
    /// During the Pre-Login handshake, the client and the server negotiate the
    /// wire encryption to be used.
    #[derive(Default)]
    pub struct Encrypt: u8 {
        /// Encryption is available but on.
        const ON = 0x01;

        /// Encryption is not available.
        const NOT_SUPPORTED = 0x02;

        /// Encryption is required.
        const REQUIRED = 0x03;

        /// The client certificate should be used to authenticate
        /// the user in place of a user/password.
        const CLIENT_CERT = 0x80;
    }
}

#[derive(Debug)]
pub struct TraceId {
    pub connection_id: Uuid,
    pub activity_id: Uuid,
    pub activity_seq: u32,
}

#[derive(Debug, Default)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub build: u16,
    pub sub_build: u16,
}

impl Version {
    fn encode(&self, buf: &mut Vec<u8>) {
        buf.push(self.major);
        buf.push(self.minor);
        buf.put_u16::<BigEndian>(self.build);
        buf.put_u16::<BigEndian>(self.sub_build);
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.build)
    }
}

#[cfg(test)]
#[test]
fn test_encode_pre_login() {
    let mut buf = Vec::new();

    let pre_login = PreLogin {
        version: Version {
            major: 9,
            minor: 0,
            build: 0,
            sub_build: 0,
        },
        encryption: Encrypt::ON,
        instance: Some(""),
        thread_id: Some(0x00000DB8),
        multiple_active_result_sets: Some(true),
        ..Default::default()
    };

    // From v20191101 of MS-TDS
    #[rustfmt::skip]
    let expected = vec![
        0x00, 0x00, 0x1A, 0x00, 0x06, 0x01, 0x00, 0x20, 0x00, 0x01, 0x02, 0x00, 0x21, 0x00, 
        0x01, 0x03, 0x00, 0x22, 0x00, 0x04, 0x04, 0x00, 0x26, 0x00, 0x01, 0xFF, 0x09, 0x00, 
        0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0xB8, 0x0D, 0x00, 0x00, 0x01
    ];

    pre_login.encode(&mut buf);

    assert_eq!(expected, buf);
}

#[cfg(test)]
#[test]
fn test_decode_pre_login() {
    #[rustfmt::skip]
    let buffer = [
        0, 0, 11, 0, 6, 1, 0, 17, 0, 1, 255,
        14, 0, 12, 209, 0, 0, 0,
    ];

    let pre_login = PreLogin::decode(&buffer[..]).unwrap();

    // v14.0.3281
    assert_eq!(pre_login.version.major, 14);
    assert_eq!(pre_login.version.minor, 0);
    assert_eq!(pre_login.version.build, 3281);
    assert_eq!(pre_login.version.sub_build, 0);

    // ENCRYPT_OFF
    assert_eq!(pre_login.encryption.bits(), 0);
}
