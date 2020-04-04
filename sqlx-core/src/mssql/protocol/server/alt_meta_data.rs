use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::type_info::TypeInfo;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// ALTMETADATA =
//  TokenType
//  Count
//  Id
//  ByCols
//  *(<ByCols> ColNum)
//  1*ComputeData
#[derive(Debug)]
pub struct AltMetaData {
    count: u16,
    id: u16,
    by_cols: u8,
    columns: Vec<(u8, u16)>,
    compute_data: Vec<ComputeData>,
}

#[derive(Debug)]
pub struct ComputeData {
    op: Op,
    operand: u16,
    user_type: u32,
    flags: Flags,
    type_info: TypeInfo,
    table_name: Option<Vec<String>>,
    col_name: String,
}

impl Decode<'_> for ComputeData {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let op = Op::decode(buf.get_u8()?)?;
        let operand = buf.get_u16::<LittleEndian>()?;
        let user_type = buf.get_u32::<LittleEndian>()?;
        let flags = Flags::from_bits_truncate(buf.get_u16::<LittleEndian>()?);
        let type_info = TypeInfo::decode(buf)?;
        let table_name = if type_info.has_table_name() {
            let num_parts = buf.get_u8()?;

            let mut parts = Vec::new();
            for _ in (0..num_parts) {
                parts.push(buf.get_utf16_us_str()?);
            }

            Some(parts)
        } else {
            None
        };

        let col_name = buf.get_utf16_b_str()?;

        Ok(Self {
            op,
            operand,
            user_type,
            flags,
            type_info,
            table_name,
            col_name,
        })
    }
}

#[derive(Debug)]
pub enum Op {
    // Standard deviation (STDEV)
    Stdev = 0x30,
    // Standard deviation of the population (STDEVP)
    Stdevp = 0x31,
    // Variance (VAR)
    Var = 0x32,
    // Variance of population (VARP)
    Varp = 0x33,
    // Count of rows (COUNT)
    Cnt = 0x4B,
    // Sum of the values in the rows (SUM)
    Sum = 0x4D,
    // Average of the values in the rows (AVG)
    Avg = 0x4F,
    // Minimum value of the rows (MIN)
    Min = 0x51,
    // Maximum value of the rows (MAX)
    Max = 0x52,
}

impl Op {
    pub fn decode(value: u8) -> crate::Result<Self> {
        Ok(match value {
            0x30 => Op::Stdev,
            0x31 => Op::Stdevp,
            0x32 => Op::Var,
            0x33 => Op::Varp,
            0x4B => Op::Cnt,
            0x4D => Op::Sum,
            0x4F => Op::Avg,
            0x51 => Op::Min,
            0x52 => Op::Max,

            op => {
                return Err(protocol_err!("unexpected value {:x} for op type", op).into());
            }
        })
    }
}

bitflags! {
    pub struct Flags: u16 {
        const NULLABLE = 0x0001;
        const CASE_SEN = 0x0002;
        const UPDATEABLE1 = 0x0004;
        const UPDATEABLE2 = 0x0008;
        const IDENITTY = 0x0010;
        const COMPUTED = 0x0020;
        const FIXED_LEN_CLR_TYPE = 0x0100;
    }
}

impl Decode<'_> for AltMetaData {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let count = buf.get_u16::<LittleEndian>()?;
        let id = buf.get_u16::<LittleEndian>()?;
        let by_cols = buf.get_u8()?;

        let mut columns = Vec::new();
        for _ in (0..by_cols) {
            let by_col = buf.get_u8()?;
            let col_num = buf.get_u16::<LittleEndian>()?;
            columns.push((by_col, col_num));
        }

        let mut compute_data = Vec::new();
        loop {
            match ComputeData::decode(buf) {
                Ok(data) => compute_data.push(data),
                Err(_) => break,
            }
        }

        Ok(Self {
            count,
            id,
            by_cols,
            columns,
            compute_data,
        })
    }
}
