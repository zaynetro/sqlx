use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::ek_info::EkInfo;
use crate::mssql::protocol::server::type_info::TypeInfo;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Describes the result setfor interpretation of following ROW data streams.
//
// Token Stream Comments:
//      The token value is 0x81
//      This token is used to tell the client the data type and length of the column data.
//      It describes the format of the data found in a ROWdata stream.
//      All COLMETADATA data streams are grouped together.
//
// Token Stream Definition:
//      COLMETADATA =
//          TokenType
//          Count
//          [CekTable]
//          NoMetaData / (1*ColumnData)
#[derive(Debug)]
pub struct ColMetaData {
    // The count of columns (number of aggregate operators) in the token stream. In the event
    // that the client requested no metadata to be returned (see section 2.2.6.6for information
    // about the OptionFlags parameter in the RPCRequest token), the value of Count will be 0xFFFF.
    // This has the same effect on Count as a zero value (for example, no ColumnData is sent).
    count: u16,
    cek_table: Option<CekTable>,
    column_data: Vec<ColumnData>,
}

// A table of various encryption keys that are used to secure the plaintext data. It contains one
// row foreach encryption key. Each row can have multiple encryption key values, and each value
// represents the cipher text of the same encryption key that is secured by using a different
// master key. The size of this table is determined by EkValueCount. This table MUST be sent when
// COLUMNENCRYPTION is negotiated by client and server and is turned on.
#[derive(Debug)]
pub struct CekTable {
    // The size of CekTable. It represents the number of entries in CekTable.
    count: u16,
    infos: Vec<EkInfo>,
}

#[derive(Debug)]
pub struct ColumnData {
    // The user type ID of the data type of the column. Depending on the TDS version that is used,
    // valid values are 0x0000 or 0x00000000, with the exceptions of data type
    // TIMESTAMP (0x0050 or 0x00000050) and alias types (greater than 0x00FF or 0x000000FF).
    user_type: u32,
    flags: Flags,
    type_info: TypeInfo,
    // The fully qualified base table name for this column. It contains the table name length and
    // table name. This exists only for text, ntext, and image columns. It specifies how many
    // parts will be returned and then repeats PartName once for each NumParts.
    table_name: Option<Vec<String>>,
    crypto_meta_data: Option<CryptoMetaData>,
    // The column name. It contains the column name length and column name.
    col_name: String,
}

// This describes the encryption metadata for a column. It contains the ordinal, the UserType, the
// TYPE_INFO (BaseTypeInfo) for the plaintext value, the encryption algorithm that is used, the
// algorithm name literal, the encryption algorithm type, and the normalization version.
#[derive(Debug)]
pub struct CryptoMetaData {
    // Where the encryption key information is located in CekTable. Ordinal starts at 0.
    ordinal: u16,
    // The user type ID of the data type of the column. Depending on the TDS version that is used,
    // valid values are 0x0000 or 0x00000000, with the exceptions of data type
    // TIMESTAMP (0x0050 or 0x00000050) and alias types (greater than 0x00FF or 0x000000FF).
    user_type: u32,
    // The TYPEINFO for the plaintext data.
    base_type_info: TypeInfo,
    // A byte that describes the encryption algorithm that is used.
    //
    // If EncryptionAlgo is set to 1, the algorithm that is used is AEAD_AES_256_CBC_HMAC_SHA512,
    // as described in [IETF-AuthEncr] section 5.4. Other values are reserved for future use.
    encryption_algo: u8,
    // Reserved for future use.
    algo_name: Option<String>,
    // The normalization version to which plaintext data MUST be normalized.
    // Version numbering starts at 0x01.
    norm_version: u8,
}

// The size of the Flags parameter is always fixed at 16 bits regardless of the TDS version.
// Each of the 16 bits of the Flags parameter is interpreted based on the TDS version
// negotiated during login.
bitflags! {
    pub struct Flags: u16 {
        // Its value is 1 if the column is nullable.
        const NULLABLE = 0x0001;
        // Set to 1 for string columns with binary collation and always for the XML data type.
        // Set to 0 otherwise.
        const CASE_SEN = 0x0002;
        // usUpdateable is a 2-bit field. Its value is 0 if column is read-only, 1 if column is
        // read/write and2 if updateable is unknown.
        const UPDATEABLE1 = 0x0004;
        const UPDATEABLE2 = 0x0008;
        // Its value is 1 if the column is an identity column.
        const IDENITTY = 0x0010;
        // Its value is 1 if the column is a COMPUTED column.
        const COMPUTED = 0x0020;
        // Its value is 1 if the column is a fixed-length common language runtime
        // user-defined type (CLR UDT).
        const FIXED_LEN_CLR_TYPE = 0x0100;
        // fSparseColumnSet, introduced in TDSversion 7.3.B, is a bit flag. Its value is 1 if the
        // column is the special XML column for the sparse column set. For information about using
        // column sets, see [MSDN-ColSets]
        const SPARSE_COLUMN_SET = 0x0200;
        // Its value is 1 if the column is encrypted transparently and
        // has to be decrypted to view the plaintext value. This flag is valid when the column
        // encryption feature is negotiated between client and server and is turned on.
        const ENCRYPTED = 0x0400;
        // Its value is 1 if the column is part of a hidden primary key created to support a
        // T-SQL SELECT statement containing FOR BROWSE.
        const HIDDEN = 0x0800;
        // Its value is 1 if the column is part of a primary key for the row
        // and the T-SQL SELECT statement contains FOR BROWSE.
        const KEY = 0x1000;
        // Its value is 1 if it is unknown whether the column might be nullable.
        const NULLABLE_UNKNOWN = 0x2000;
    }
}

impl Decode<'_> for CryptoMetaData {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let ordinal = buf.get_u16::<LittleEndian>()?;
        let user_type = buf.get_u32::<LittleEndian>()?;
        let base_type_info = TypeInfo::decode(buf)?;
        let encryption_algo = buf.get_u8()?;
        let algo_name = None;
        let norm_version = buf.get_u8()?;

        Ok(Self {
            ordinal,
            user_type,
            base_type_info,
            encryption_algo,
            algo_name,
            norm_version,
        })
    }
}

impl Decode<'_> for ColumnData {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let user_type = buf.get_u32::<LittleEndian>()?;
        let flags = Flags::from_bits_truncate(buf.get_u16::<LittleEndian>()?);
        let type_info = TypeInfo::decode(buf)?;
        let table_name = if type_info.r#type.has_table_name() {
            let num_parts = buf.get_u8()?;
            let mut parts = Vec::new();

            for _ in (0..num_parts) {
                parts.push(buf.get_utf16_us_str()?);
            }

            Some(parts)
        } else {
            None
        };

        let crypto_meta_data = if flags.contains(Flags::ENCRYPTED) {
            Some(CryptoMetaData::decode(buf)?)
        } else {
            None
        };

        let col_name = buf.get_utf16_b_str()?;

        Ok(Self {
            user_type,
            flags,
            type_info,
            table_name,
            crypto_meta_data,
            col_name,
        })
    }
}

impl Decode<'_> for CekTable {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let count = buf.get_u16::<LittleEndian>()?;
        let mut infos = Vec::new();
        for _ in (0..count) {
            infos.push(EkInfo::decode(buf)?);
        }

        Ok(Self { count, infos })
    }
}

impl Decode<'_> for ColMetaData {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let count = buf.get_u16::<LittleEndian>()?;

        let cek_table = None;
        // let cek_table = Some(CekTable::decode(buf)?);

        let column_data = if buf[0] == 0xFF && buf[1] == 0xFF {
            Vec::new()
        } else {
            let mut data = Vec::new();

            loop {
                match ColumnData::decode(buf) {
                    Ok(v) => data.push(v),
                    Err(_) => break,
                }
            }

            data
        };

        Ok(Self {
            count,
            cek_table,
            column_data,
        })
    }
}
