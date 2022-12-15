use std::str::FromStr;

use crate::modules::native;
use crate::{db_helpers, errors, types};

// TODO PHASE 2 pagination by artificial index added to balance_changes
pub(crate) async fn get_near_history(
    balances_pool: &sqlx::Pool<sqlx::Postgres>,
    account_id: &near_primitives::types::AccountId,
    pagination: &types::query_params::HistoryPagination,
) -> crate::Result<Vec<native::schemas::HistoryItem>> {
    let query = r"
        SELECT
            involved_account_id,
            delta_nonstaked_amount + delta_staked_amount delta_balance,
            absolute_nonstaked_amount + absolute_staked_amount balance,
            cause,
            status,
            block_timestamp block_timestamp_nanos
        FROM balance_changes
        WHERE affected_account_id = $1 AND block_timestamp < $2::numeric(20, 0)
        ORDER BY block_timestamp DESC
        LIMIT $3::numeric(20, 0)
    ";

    let history_info = db_helpers::select_retry_or_panic::<super::models::NearHistoryInfo>(
        balances_pool,
        query,
        &[
            account_id.to_string(),
            pagination.block_timestamp.to_string(),
            pagination.limit.to_string(),
        ],
    )
    .await?;

    let mut result: Vec<native::schemas::HistoryItem> = vec![];
    for history in history_info {
        result.push(history.try_into()?);
    }
    Ok(result)
}

impl TryFrom<super::models::NearHistoryInfo> for native::schemas::HistoryItem {
    type Error = errors::Error;

    fn try_from(info: super::models::NearHistoryInfo) -> crate::Result<Self> {
        let involved_account_id: Option<types::AccountId> =
            if let Some(account_id) = info.involved_account_id {
                Some(near_primitives::types::AccountId::from_str(&account_id)?.into())
            } else {
                None
            };
        Ok(Self {
            involved_account_id,
            delta_balance: info.delta_balance.to_string(),
            balance: types::numeric::to_u128(&info.balance)?.into(),
            cause: info.cause,
            status: info.status,
            metadata: super::get_near_metadata(),
            block_timestamp_nanos: types::numeric::to_u64(&info.block_timestamp_nanos)?.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;

    #[tokio::test]
    async fn test_near_history() {
        let block = get_block();
        // Using the other pool because we have this table at the other DB
        dotenv::dotenv().ok();
        let url_balances =
            &std::env::var("DATABASE_URL_BALANCES").expect("failed to get database url");
        let pool = sqlx::PgPool::connect(url_balances)
            .await
            .expect("failed to connect to the balances database");
        let account = near_primitives::types::AccountId::from_str("vasya.near").unwrap();
        let pagination = types::query_params::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let balance = get_near_history(&pool, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_near_history_with_failed_receipts() {
        let block = db_helpers::Block {
            timestamp: 1618591017607373869,
            height: 34943083,
        };
        // Using the other pool because we have this table at the other DB
        dotenv::dotenv().ok();
        let url_balances =
            &std::env::var("DATABASE_URL_BALANCES").expect("failed to get database url");
        let pool = sqlx::PgPool::connect(url_balances)
            .await
            .expect("failed to connect to the balances database");
        let account = near_primitives::types::AccountId::from_str("zubkowi.near").unwrap();
        let pagination = types::query_params::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let balance = get_near_history(&pool, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }
}
