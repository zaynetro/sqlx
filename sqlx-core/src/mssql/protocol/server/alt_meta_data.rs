use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::type_info::TypeInfo;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Describes the data type, length, and name of column data that result from a SQL statement
//      that generates totals.
//
// Token Stream Comments:
//      The token value is 0x88.
//      This token is used to tell the client the data type and length of the column data.
//      It describes the format of the data found in an ALTROW data stream. ALTMETADATA and the
//      corresponding ALTROW MUST be in the same result set.
//
// All ALTMETADATA data streams are grouped.
//
// A preceding COLMETADATA MUST exist before an ALTMETADATA token. There might be COLINFO and
// TABNAME streams between COLMETADATA and ALTMETADATA.
//
// Note: ALTMETADATA was deprecated in TDS 7.4.
//
// ALTMETADATA =
//      TokenType
//      Count
//      Id
//      ByCols
//      *(<ByCols> ColNum)
//      1*ComputeData
#[derive(Debug)]
pub struct AltMetaData {
    // The count of columns (number of aggregate operators) in the token stream.
    count: u16,
    // The Id of the SQL statement to which the total column formats apply. Each ALTMETADATA token
    // MUST have its own unique Id in the same result set. This Id lets the client correctly
    // interpret later ALTROW data streams.
    id: u16,
    // The number of grouping columns in the SQL statement that generates totals. For example,
    // the SQL clause compute count(sales) by year, month, division, departmenthas four grouping columns.
    by_cols: u8,
    // USHORT specifying the column number as it appears in the COMPUTE clause. ColNum appears ByCols times.
    columns: Vec<(u8, u16)>,
    compute_data: Vec<ComputeData>,
}

#[derive(Debug)]
pub struct ComputeData {
    // The type of aggregate operator.
    op: Op,
    // The column number, starting from 1, in the result set that is the operand to the aggregate operator.
    operand: u16,
    // The user type ID of the data type of the column. Depending on the TDS version that is used,
    // valid values are 0x0000 or 0x00000000, with the exceptions of data type
    // TIMESTAMP (0x0050 or 0x00000050) and alias types (greater than 0x00FF or 0x000000FF).
    user_type: u32,
    // With the exception of fNullable, all of these bit flags SHOULD be set to zero.
    flags: Flags,
    type_info: TypeInfo,
    // See section 2.2.7.4 for a description of TableName. This field SHOULD never be sent because
    // SQL statements that generate totals exclude NTEXT/TEXT/IMAGE.
    table_name: Option<Vec<String>>,
    // The column name. Contains the column name length and column name.
    col_name: String,
}

impl Decode<'_> for ComputeData {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let op = Op::decode(buf.get_u8()?)?;
        let operand = buf.get_u16::<LittleEndian>()?;
        let user_type = buf.get_u32::<LittleEndian>()?;
        let flags = Flags::from_bits_truncate(buf.get_u16::<LittleEndian>()?);
        let type_info = TypeInfo::decode(buf)?;

        // TableName should never be sent.
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
