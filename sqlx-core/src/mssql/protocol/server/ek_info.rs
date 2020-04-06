use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::type_info::TypeInfo;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// The EK_INFO rule applies to messages that have encrypted values and describes the encryption key
// information. The encryption key information includes the various encryption key values that are
// obtained by securing an encryption key by using different master keys. This rule applies only
// if the column encryption feature is negotiated by the client and the server and is turned ON.
#[derive(Debug)]
pub struct EkInfo {
    // A 4 byte integer value that represents the database ID where the column encryption key is stored.
    database_id: u32,
    // An identifier for the column encryption key.
    cek_id: u32,
    // The key version of the column encryption key.
    cek_version: u32,
    // The metadata version for the column encryption key.
    cek_md_version: u64,
    // The metadata and encrypted value that describe an encryption key. This is enough information
    // to allow retrieval of plaintext encryption keys.
    key_values: Vec<EncryptionKeyValue>,
}

#[derive(Debug)]
pub struct EncryptionKeyValue {
    // The ciphertext containing the encryption key that is secured with the master.
    encryption_key: String,
    // The key store name component of the location where the master key is saved.
    key_store_name: String,
    // The key path component of the location where the master key is saved.
    key_path: String,
    // The name of the algorithm that is used for encrypting the encryption key.
    asymmetric_algo: String,
}

impl Decode<'_> for EncryptionKeyValue {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let encryption_key = buf.get_utf16_us_str()?;
        let key_store_name = buf.get_utf16_b_str()?;
        let key_path = buf.get_utf16_us_str()?;
        let asymmetric_algo = buf.get_utf16_b_str()?;

        Ok(Self {
            encryption_key,
            key_store_name,
            key_path,
            asymmetric_algo,
        })
    }
}

impl Decode<'_> for EkInfo {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let database_id = buf.get_u32::<LittleEndian>()?;
        let cek_id = buf.get_u32::<LittleEndian>()?;
        let cek_version = buf.get_u32::<LittleEndian>()?;
        let cek_md_version = buf.get_u64::<LittleEndian>()?;

        // The count of EncryptionKeyValue elements that are present in the message.
        let count = buf.get_u8()?;
        let mut key_values = Vec::new();
        for _ in (0..count) {
            key_values.push(EncryptionKeyValue::decode(buf)?);
        }

        Ok(Self {
            database_id,
            cek_id,
            cek_version,
            cek_md_version,
            key_values,
        })
    }
}
