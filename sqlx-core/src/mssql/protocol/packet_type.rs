#[derive(Debug, Copy, Clone)]
pub enum PacketType {
    SqlBatch = 1,
    PreTds7Login = 2,
    Rpc = 3,
    TabularResult = 4,
    AttentionSignal = 6,
    BulkLoadData = 7,
    FederatedAuthToken = 8,
    TransactionManagerRequest = 14,
    Tds7Login = 16,
    Sspi = 17,
    PreLogin = 18,
}

impl PacketType {
    pub fn decode(value: u8) -> crate::Result<Self> {
        Ok(match value {
            1 => PacketType::SqlBatch,
            2 => PacketType::PreTds7Login,
            3 => PacketType::Rpc,
            4 => PacketType::TabularResult,
            6 => PacketType::AttentionSignal,
            7 => PacketType::BulkLoadData,
            8 => PacketType::FederatedAuthToken,
            14 => PacketType::TransactionManagerRequest,
            16 => PacketType::Tds7Login,
            17 => PacketType::Sspi,
            18 => PacketType::PreLogin,

            ty => {
                return Err(protocol_err!("unexpected value {:x} for packet type", ty).into());
            }
        })
    }
}
