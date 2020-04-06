use crate::io::Buf;
use crate::mssql::io::BufExt;
use crate::mssql::protocol::server::ek_info::EkInfo;
use crate::mssql::protocol::server::type_info::TypeInfo;
use crate::mssql::protocol::Decode;
use bitflags::bitflags;
use byteorder::{BigEndian, LittleEndian};

// Token Stream Function:
//      Introduced inTDS 7.4, the DATACLASSIFICATION token SHOULD<39>describe the data
//      classification of the query result set.
//
// Token Stream Comments:
//      The token value is 0xA3.
//      This token only be sent by the server if the client sends a DATACLASSIFICATION FeatureExt
//      in the Login message and the server responds with a DATACLASSIFICATION FeatureExtAck.  When
//      this token is used, the token is sent by the server in response to every SQLBatch request.
//
// Token Stream Definition:
//      DATACLASSIFICATION =
//          TokenType
//          SensitivityLabels
//          InformationTypes
//          [SensitivityRank]
//          DataClassificationPerColumnData
#[derive(Debug)]
pub struct DataClassification {
    sensitivity_labels: Vec<SensitivityLabel>,
    information_types: Vec<InformationType>,
    // A relative ranking of the sensitivity of a query or of a column that is part of per-column data.
    // A sensitivity ranking is sent by the server only if both of the following are true:
    //      - The client sends a DATACLASSIFICATION feature extension in a Login message in which
    //        DATACLASSIFICATION_VERSION is set to 2.
    //      - The server responds with a DATACLASSIFICATION feature extension acknowledgement in
    //        which DATACLASSIFICATION_VERSION is set to 2.
    sensitivity_rank: Option<SensitivityRank>,
    data_classification_per_column_data: Vec<Vec<SensitivityProperty>>,
}

impl Decode<'_> for DataClassification {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let count = buf.get_u16::<LittleEndian>()?;
        let mut sensitivity_labels = Vec::new();
        for _ in (0..count) {
            sensitivity_labels.push(SensitivityLabel::decode(buf)?);
        }

        let count = buf.get_u16::<LittleEndian>()?;
        let mut information_types = Vec::new();
        for _ in (0..count) {
            information_types.push(InformationType::decode(buf)?);
        }

        let sensitivity_rank = None;
        // let sensitivity_rank = Some(SensitivityRank::from(buf.get_i32::<LittleEndian>()?));

        let mut data_classification_per_column_data = Vec::new();
        let count = buf.get_u16::<LittleEndian>()?;
        for _ in (0..count) {
            let inner_count = buf.get_u16::<LittleEndian>()?;
            let mut data_classification = Vec::new();
            for _ in (0..inner_count) {
                data_classification.push(SensitivityProperty::decode(buf)?);
            }
            data_classification_per_column_data.push(data_classification)
        }

        Ok(Self {
            sensitivity_labels,
            information_types,
            sensitivity_rank,
            data_classification_per_column_data,
        })
    }
}

#[derive(Debug)]
pub struct SensitivityLabel {
    // The name for a sensitivity label. It contains the sensitivity label name length and
    // sensitivity label name. It is intended to be human readable.
    name: String,
    // The identifier for a sensitivity label. It contains the sensitivity label identifier length
    // and sensitivity label identifier. It is intended for linking the sensitivity label to an
    // information protection system.
    id: String,
}

impl Decode<'_> for SensitivityLabel {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let name = buf.get_utf16_b_str()?;
        let id = buf.get_utf16_b_str()?;

        Ok(Self { name, id })
    }
}

#[derive(Debug)]
pub struct InformationType {
    // The name for an information type. It contains the information type name length and
    // information type name. It is intended to be human readable.
    name: String,
    // The identifier for an information type. It contains the information type identifier length
    // and information type identifier. It is intended for linking the information type to an
    // information protection system
    id: String,
}

impl Decode<'_> for InformationType {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let name = buf.get_utf16_b_str()?;
        let id = buf.get_utf16_b_str()?;

        Ok(Self { name, id })
    }
}

#[derive(Debug)]
pub struct SensitivityProperty {
    // The index into the SensitivityLabels array that indicates which SensitivityLabel is
    // associated with SensitivityProperty.A value of USHORT_MAX (0xFFFF) indicates that there is
    // no sensitivity label for SensitivityProperty.
    label_index: u16,
    // The index into the InformationTypes array that indicates which InformationType is associated
    // with SensitivityProperty. A value of USHORT_MAX (0xFFFF) indicates that there is no
    // information type for SensitivityProperty.
    type_index: u16,
    rank: Option<SensitivityRank>,
}

impl Decode<'_> for SensitivityProperty {
    fn decode(mut buf: &[u8]) -> crate::Result<Self> {
        let label_index = buf.get_u16::<LittleEndian>()?;
        let type_index = buf.get_u16::<LittleEndian>()?;
        let rank = None;
        // let rank = Some(SensitivyRank::from(buf.get_i32::<LittleEndian>()?));

        Ok(Self {
            label_index,
            type_index,
            rank,
        })
    }
}

// A relative ranking of the sensitivity of a query or of a column that is part of per-column data.
// A sensitivity ranking is sent by the server only if both of the following are true:
//      - The client sends a DATACLASSIFICATION feature extension in a Login message in which
//        DATACLASSIFICATION_VERSION is set to 2.
//      - The server responds with a DATACLASSIFICATION feature extension acknowledgement in
//        which DATACLASSIFICATION_VERSION is set to 2.
#[derive(Debug)]
pub enum SensitivityRank {
    NotDefined = -1,
    None = 0,
    Low = 10,
    Medium = 20,
    High = 30,
    Critical = 40,
}

impl From<i32> for SensitivityRank {
    fn from(value: i32) -> Self {
        match value {
            -1 => SensitivityRank::NotDefined,
            0 => SensitivityRank::None,
            10 => SensitivityRank::Low,
            20 => SensitivityRank::Medium,
            30 => SensitivityRank::High,
            40 => SensitivityRank::Critical,
            v => panic!(
                "Unexpected SensitivityRank value, expected -1, 0, 10, 20, 30, 40, but got {:?}",
                v
            ),
        }
    }
}
