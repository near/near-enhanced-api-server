use crate::BigDecimal;

#[derive(sqlx::FromRow)]
pub(crate) struct AccountChangesBalance {
    pub nonstaked: BigDecimal,
    pub staked: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct NearHistoryInfo {
    pub involved_account_id: Option<String>,
    pub delta_balance: BigDecimal,
    pub delta_available_balance: BigDecimal,
    pub delta_staked_balance: BigDecimal,
    pub total_balance: BigDecimal,
    pub available_balance: BigDecimal,
    pub staked_balance: BigDecimal,
    pub cause: String,
    // pub index: super::types::U128,
    pub block_timestamp_nanos: BigDecimal,
    // pub block_height: super::types::U64,
}

#[derive(sqlx::FromRow)]
pub(crate) struct FtHistoryInfo {
    pub block_height: BigDecimal,
    pub block_timestamp: BigDecimal,
    pub amount: BigDecimal,
    pub event_kind: String,
    pub old_owner_id: String,
    pub new_owner_id: String,
}
