use crate::io::BufMut;
use crate::mssql::io::BufMutExt;
use crate::mssql::protocol::{Encode, PacketType};
use byteorder::LittleEndian;

const HEADER_TRANSACTION_DESCRIPTOR: u16 = 0x00_02;

// SQLBatch = ALL_HEADERS
//            *EnclavePacket
//            SQLText
#[derive(Debug)]
pub struct SqlBatch<'a> {
    pub(crate) sql: &'a str,
}

impl Encode for SqlBatch<'_> {
    fn r#type() -> PacketType {
        PacketType::SqlBatch
    }

    fn encode(&self, buf: &mut Vec<u8>) {
        // ALL_HEADERS -> TotalLength
        buf.put_u32::<LittleEndian>(4 + 18); // 4 + 18

        // [Header] Transaction Descriptor
        //  SQL_BATCH messages require this header
        //  contains information regarding number of outstanding requests for MARS
        buf.put_u32::<LittleEndian>(18); // 4 + 2 + 8 + 4
        buf.put_u16::<LittleEndian>(HEADER_TRANSACTION_DESCRIPTOR);

        // [TransactionDescriptor] a number that uniquely identifies the current transaction
        // TODO: use this once we support transactions, it will be given to us from the
        //       server ENVCHANGE event
        buf.put_u64::<LittleEndian>(0);

        // [OutstandingRequestCount] Number of active requests to MSSQL from the
        //                           same connection
        // NOTE: Long-term when we support MARS we need to connect this value correctly
        buf.put_u32::<LittleEndian>(1);

        // SQLText
        buf.put_utf16_str(self.sql);
    }
}
