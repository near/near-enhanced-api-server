use crate::BigDecimal;

#[derive(sqlx::FromRow)]
pub(crate) struct FtHistoryInfo {
    // TODO PHASE 2 add symbol
    // pub block_height: BigDecimal,
    pub block_timestamp: BigDecimal,
    pub amount: BigDecimal,
    pub cause: String,
    pub status: String,
    pub old_owner_id: String,
    pub new_owner_id: String,
}
