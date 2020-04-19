use std::io;

use byteorder::{ByteOrder, LittleEndian};
use subslice::SubsliceExt;

use crate::io::Buf;
use crate::mssql::protocol::server::type_info::TypeInfo;

pub trait BufExt<'a>: Buf<'a> {
    // get a UTF-16 string
    fn get_utf16_str(&mut self, mut n: usize) -> io::Result<String> {
        let mut raw = Vec::with_capacity(n * 2);

        while n > 0 {
            let ch = self.get_u16::<LittleEndian>()?;
            raw.push(ch);
            n -= 1;
        }

        String::from_utf16(&raw).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
    }

    // [B_VARCHAR] get a UTF-16 string prefixed with a single byte for the length
    fn get_utf16_b_str(&mut self) -> io::Result<String> {
        let size = self.get_u8()?;
        self.get_utf16_str(size as usize)
    }

    // [US_VARCHAR] get a UTF-16 string prefixed with a u16 for the length
    fn get_utf16_us_str(&mut self) -> io::Result<String> {
        let size = self.get_u16::<LittleEndian>()?;
        self.get_utf16_str(size as usize)
    }

    // [B_VARBYTE] get a variable amount of bytes with a single byte at the beginning the length
    fn get_b_bytes(&mut self) -> io::Result<&'a [u8]> {
        let size = self.get_u8()?;
        self.get_bytes(size as usize)
    }

    fn get_type_var_byte(&mut self, info: &TypeInfo) -> io::Result<&[u8]>;
}

impl<'a> BufExt<'a> for &'a [u8] {
    fn get_type_var_byte(&mut self, info: &TypeInfo) -> io::Result<&[u8]> {
        // TODO: PLP_BODY
        Ok(if info.r#type.is_fixed_len() {
            // ([TYPE_VARBYTE] *BYTE) where TYPE_VARBYTE is *not* present
            let len = info.r#type.fixed_len();
            &self[0..len]
        } else if info.r#type.is_var_len() {
            // ([TYPE_VARBYTE] *BYTE) where TYPE_VARBYTE *is* present
            let len = if info.r#type.is_bytelen() {
                self.get_u8()? as u32
            } else if info.r#type.is_ushort_len() {
                self.get_u16::<LittleEndian>()? as u32
            } else {
                self.get_u32::<LittleEndian>()?
            };

            &self[0..len as usize]
        } else if info.r#type.is_charbin_null() {
            // CHARBIN_NULL
            let len = self.find(&[0, 0, 0, 0]).unwrap();
            let value = &self[0..len];
            self.advance(4);
            value
        } else {
            // GEN_NULL
            let len = memchr::memchr(0, &self).unwrap();
            let value = &self[0..len];
            self.advance(1);
            value
        })
    }
}
