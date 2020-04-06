use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::client::pre_login::Version;
use crate::mssql::protocol::server::col_meta_data::{CryptoMetaData, Flags};
use crate::mssql::protocol::server::type_info::TypeInfo;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Used to send the return value of an RPC to the client. When an RPC is executed, the
//      associated parameters might be defined as input or output (or "return") parameters.
//      This token is used to send a description of the return parameter to the client.
//      This token is also used to describe the value returned by a UDF when executed as an RPC.
//
// Token Stream Comments:
//      - The token value is 0xAC.
//      - Multiple return values can exist per RPC. There is a separate RETURNVALUE token sent
//        for each parameter returned.
//      - Large Object output parameters are reordered to appear at the end of the stream.
//        First the group of small parameters is sent, followed by the group of large
//        output parameters. There is no reordering within the groups.
//      - A UDF cannot have return parameters. As such, if a UDF is executed as an RPC there is
//        exactly one RETURNVALUE token sent to the client.
//
// Token Stream Definition:
//      RETURNVALUE      =  TokenType
//                          ParamOrdinal
//                          ParamName
//                          Status
//                          UserType
//                          Flags
//                          TypeInfo
//                          CryptoMetadata
//                          Value
#[derive(Debug)]
pub struct ReturnValue {
    // Indicates the ordinal position of the output parameter in the original RPC call. Large
    // Object output parameters are reordered to appear at the end of the stream. First the group
    // of small parameters is sent, followed by the group of large output parameters. There is no
    // reordering within the groups.
    param_ordinal: u16,
    // The parameter name length and parameter name (within B_VARCHAR).
    param_name: String,
    status: Status,
    // The user type ID of the data type of the column. Depending on the TDS version that is used,
    // valid values are 0x0000 or 0x00000000, with the exceptions of data type
    // TIMESTAMP (0x0050 or 0x00000050) and alias types (greater than 0x00FF or 0x000000FF).
    user_type: u32,
    flags: Flags,
    // The TYPE_INFO for the message.
    type_info: TypeInfo,
    crypto_meta_data: CryptoMetaData,
    // TODO:
    // The type-dependent data for the parameter (within TYPE_VARBYTE)
    // value: TYPE_VARBYTE,
}

bitflags! {
    pub struct Status: u8 {
        // If ReturnValue corresponds to OUTPUT parameter of a stored procedureinvocation.
        const STORED_PROCEDURE = 0x01;
        // If ReturnValue corresponds to return value of User Defined Function.
        const USER_DEFINED_FUNCTION = 0x02;
    }
}

impl Decode<'_> for ReturnValue {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let param_ordinal = buf.get_u16::<LittleEndian>()?;
        let param_name = buf.get_utf16_b_str()?;
        let status = Status::from_bits_truncate(buf.get_u8()?);
        let user_type = buf.get_u32::<LittleEndian>()?;
        let flags = Flags::from_bits_truncate(buf.get_u16::<LittleEndian>()?);
        let type_info = TypeInfo::decode(buf)?;
        let crypto_meta_data = CryptoMetaData::decode(buf)?;

        if buf.get_u8()? != 0x00 {
            unimplemented!("TYPE_VARBYTE is unimplemented");
        }

        Ok(Self {
            param_ordinal,
            param_name,
            status,
            user_type,
            flags,
            type_info,
            crypto_meta_data,
        })
    }
}
