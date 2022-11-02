use crate::BigDecimal;

#[derive(sqlx::FromRow)]
pub(crate) struct AccountChangesBalance {
    pub balance: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct CoinHistoryInfo {
    // TODO PHASE 2 add symbol
    pub event_index: BigDecimal,
    pub involved_account_id: Option<String>,
    pub delta_balance: BigDecimal,
    pub balance: BigDecimal,
    pub cause: String,
    pub status: String,
    pub block_timestamp_nanos: BigDecimal,
    pub block_height: BigDecimal,
}
