use crate::BigDecimal;

#[derive(sqlx::FromRow)]
pub(crate) struct NftHistoryInfo {
    // pub index: super::types::U128,
    pub cause: String,
    pub status: String,
    pub old_account_id: String,
    pub new_account_id: String,
    pub block_timestamp_nanos: BigDecimal,
    pub block_height: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct NftCount {
    pub contract_id: String,
    pub count: i64,
    pub last_updated_at_timestamp: BigDecimal,
}
