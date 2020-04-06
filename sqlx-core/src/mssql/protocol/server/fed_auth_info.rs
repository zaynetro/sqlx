use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Introduced in TDS 7.4, federated authenticationinformation is returned to the client to be
//      used for generating a Federated Authentication Token during the login process. This token
//      MUST be the only token in a Federated Authentication Information message and MUST NOT be
//      included in any other message type.
//
// Token Stream Comments:
//      The token value is 0xEE.
//
// Token Stream Definition:
//      FEDAUTHINFO            = TokenType           ; (introduced in TDS 7.4)
//                               TokenLength
//                               CountOfInfoIDs
//                               1*FedAuthInfoOpt
//                               FedAuthInfoData
#[derive(Debug)]
pub struct FedAuthInfo<'a> {
    // The length of the whole Federated Authentication Information token, not including the size
    // occupied by TokenLength itself. The minimum value for this field is sizeof(DWORD) because
    // the field CountOfInfoIDs MUST be present even if no federated authentication information
    // is sent as part of the token.
    token_length: u32,
    // The number of federated authentication information options that are sent in the token.
    // If no FedAuthInfoOpt is sent in the token, this field MUST be present and set to 0.
    count_of_info_ids: u32,
    options: Vec<FedAuthInfoOpt>,
    // The actual information data as binary, with the length in bytes equal to FedAuthInfoDataLen.
    data: &'a [u8],
}

#[derive(Debug)]
pub struct FedAuthInfoOpt {
    // The unique identifier number for the type of information.
    id: u8,
    // The length of FedAuthInfoData, in bytes.
    len: u32,
    // The offset at which the federated authentication information data for FedAuthInfoID is
    // present, measured from the address of CountOfInfoIDs.
    offset: u32,
}

impl Decode<'_> for FedAuthInfoOpt {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let id = buf.get_u8()?;
        let len = buf.get_u32::<LittleEndian>()?;
        let offset = buf.get_u32::<LittleEndian>()?;

        Ok(Self { id, len, offset })
    }
}

impl<'a> Decode<'a> for FedAuthInfo<'a> {
    fn decode(mut buf: &'a [u8]) -> crate::Result<Self> {
        let token_length = buf.get_u32::<LittleEndian>()?;
        let count_of_info_ids = buf.get_u32::<LittleEndian>()?;

        let mut options = Vec::new();
        for _ in (0..count_of_info_ids) {
            options.push(FedAuthInfoOpt::decode(buf)?);
        }

        Ok(Self {
            token_length,
            count_of_info_ids,
            options,
            data: &buf[..],
        })
    }
}
