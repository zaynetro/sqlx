use byteorder::{ByteOrder, LittleEndian};

use crate::io::BufMut;

pub trait BufMutExt: BufMut {
    fn put_utf16_str(&mut self, s: &str) {
        let mut enc = s.encode_utf16();
        while let Some(ch) = enc.next() {
            self.put_u16::<LittleEndian>(ch);
        }
    }
}

impl<T: BufMut> BufMutExt for T {}
