use std::io;

use byteorder::{ByteOrder, LittleEndian};

use crate::io::Buf;

pub trait BufExt<'a>: Buf<'a> {
    // get a UTF-16 string
    fn get_utf16_str(&mut self, mut n: usize) -> io::Result<String> {
        let mut raw = Vec::with_capacity(n * 2);

        let orig = n.clone();
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
}

impl<'a> BufExt<'a> for &'a [u8] {}
