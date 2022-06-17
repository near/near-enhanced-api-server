use crate::BigDecimal;

#[derive(sqlx::FromRow)]
pub(crate) struct AccountChangesBalance {
    pub nonstaked: BigDecimal,
    pub staked: BigDecimal,
    pub block_timestamp: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct BlockTimestamp {
    pub block_timestamp: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct ActionKind {
    pub action_kind: String,
}
