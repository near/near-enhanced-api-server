use crate::BigDecimal;

#[derive(sqlx::FromRow)]
pub(crate) struct AccountChangesBalance {
    pub balance: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct NearHistoryInfo {
    pub involved_account_id: Option<String>,
    pub delta_balance: BigDecimal,
    pub balance: BigDecimal,
    pub cause: String,
    pub status: String,
    // pub index: super::types::U128,
    pub block_timestamp_nanos: BigDecimal,
    // pub block_height: super::types::U64,
}

#[derive(sqlx::FromRow)]
pub(crate) struct CoinHistoryInfo {
    // TODO PHASE 2 add symbol
    // pub block_height: BigDecimal,
    pub block_timestamp: BigDecimal,
    pub amount: BigDecimal,
    pub cause: String,
    pub status: String,
    pub old_owner_id: String,
    pub new_owner_id: String,
}
