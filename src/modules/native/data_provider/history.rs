use std::str::FromStr;

use crate::modules::native;
use crate::{db_helpers, errors, types};

pub(crate) async fn get_near_history(
    db_helpers::BalancesPool(pool_balances): &db_helpers::BalancesPool,
    account_id: &near_primitives::types::AccountId,
    block: &db_helpers::Block,
    pagination: &types::query_params::Pagination,
) -> crate::Result<Vec<native::schemas::HistoryItem>> {
    let after_event_index = if let Some(index) = pagination.after_event_index {
        index
    } else {
        // +1 because we need to include given timestamp to result. Query has strict less operator
        db_helpers::timestamp_to_event_index(block.timestamp + 1)
    };

    let query = r"
        SELECT
            event_index,
            involved_account_id,
            delta_nonstaked_amount + delta_staked_amount delta_balance,
            absolute_nonstaked_amount + absolute_staked_amount balance,
            cause,
            status,
            block_timestamp block_timestamp_nanos,
            block_height
        FROM near_balance_events
        WHERE affected_account_id = $1 AND event_index < $2::numeric(38, 0)
           AND event_index >= 16619903149406361800000000000000000 -- todo drop this when we finish collecting the data
        ORDER BY event_index DESC
        LIMIT $3::numeric(20, 0)
    ";

    let history_info = db_helpers::select_retry_or_panic::<super::models::NearHistoryInfo>(
        pool_balances,
        query,
        &[
            account_id.to_string(),
            after_event_index.to_string(),
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

    fn try_from(history: super::models::NearHistoryInfo) -> crate::Result<Self> {
        let involved_account_id: Option<types::AccountId> =
            if let Some(account_id) = history.involved_account_id {
                Some(near_primitives::types::AccountId::from_str(&account_id)?.into())
            } else {
                None
            };
        Ok(Self {
            event_index: types::numeric::to_u128(&history.event_index)?.into(),
            involved_account_id,
            delta_balance: history.delta_balance.to_string(),
            balance: types::numeric::to_u128(&history.balance)?.into(),
            cause: history.cause,
            status: history.status,
            metadata: super::get_near_metadata(),
            block_timestamp_nanos: types::numeric::to_u64(&history.block_timestamp_nanos)?.into(),
            block_height: types::numeric::to_u64(&history.block_height)?.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;

    #[tokio::test]
    async fn test_near_history() {
        let pool_balances = init_balances_db().await;
        let account = near_primitives::types::AccountId::from_str("cvirkun.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: None,
        };
        let block = get_block();

        let history = get_near_history(&pool_balances, &account, &block, &pagination).await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_near_history_next_page() {
        let pool_explorer = init_explorer_db().await;
        let pool_balances = init_balances_db().await;
        let account = near_primitives::types::AccountId::from_str("cvirkun.near").unwrap();
        let index = 16708676458550339330000000000000003;
        let pagination = types::query_params::Pagination {
            limit: 3,
            after_event_index: Some(index),
        };
        let block = db_helpers::get_block_from_pagination(&pool_explorer, &pagination)
            .await
            .unwrap();

        let history = get_near_history(&pool_balances, &account, &block, &pagination).await;
        insta::assert_debug_snapshot!(history);
        assert!(
            history.unwrap().first().unwrap().event_index.0 < index,
            "Next page should not include event from previous page"
        );
    }

    #[tokio::test]
    async fn test_near_history_with_failed_receipts() {
        let pool_explorer = init_explorer_db().await;
        let pool_balances = init_balances_db().await;
        let account = near_primitives::types::AccountId::from_str("aurora").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: Some(16708676800272181160000000010000005),
        };
        let block = db_helpers::get_block_from_pagination(&pool_explorer, &pagination)
            .await
            .unwrap();

        let history = get_near_history(&pool_balances, &account, &block, &pagination).await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_near_history_account_never_existed() {
        let pool_balances = init_balances_db().await;
        let account =
            near_primitives::types::AccountId::from_str("two-idiots-and-a-half.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 10,
            after_event_index: None,
        };
        let block = get_block();

        let history = get_near_history(&pool_balances, &account, &block, &pagination)
            .await
            .unwrap();
        assert!(history.is_empty());
    }

    // todo return this when we collect all the history
    // #[tokio::test]
    // async fn test_near_history_account_deleted() {
    //     let pool_balances = init_balances_db().await;
    //     let account =
    //         near_primitives::types::AccountId::from_str("tezeract.near").unwrap();
    //     let pagination = types::query_params::Pagination {
    //         limit: 5,
    //         after_event_index: None,
    //     };
    //     let block = get_block();
    //
    //     let history = get_near_history(&pool_balances, &account, &block, &pagination)
    //         .await
    //         .unwrap();
    //     // we still show the history
    //     insta::assert_debug_snapshot!(history);
    // }
}
