use num_traits::Signed;
use std::str::FromStr;

use crate::modules::coin;
use crate::{db_helpers, errors, types, BigDecimal};

pub(crate) async fn get_near_history(
    balances_pool: &sqlx::Pool<sqlx::Postgres>,
    account_id: &near_primitives::types::AccountId,
    block: &db_helpers::Block,
    pagination: &types::query_params::Pagination,
) -> crate::Result<Vec<coin::schemas::HistoryItem>> {
    // todo
    let query = if pagination.after_event_index.is_some() {
        r"
        SELECT
            event_index,
            involved_account_id,
            delta_nonstaked_amount + delta_staked_amount delta_balance,
            absolute_nonstaked_amount + absolute_staked_amount balance,
            cause,
            status,
            block_timestamp block_timestamp_nanos,
            block_height
        FROM balance_changes
        WHERE affected_account_id = $1 AND event_index < $2::numeric(38, 0)
        ORDER BY event_index DESC
        LIMIT $4::numeric(20, 0)
    "
    } else {
        r"
        SELECT
            event_index,
            involved_account_id,
            delta_nonstaked_amount + delta_staked_amount delta_balance,
            absolute_nonstaked_amount + absolute_staked_amount balance,
            cause,
            status,
            block_timestamp block_timestamp_nanos,
            block_height
        FROM balance_changes
        WHERE affected_account_id = $1 AND block_timestamp <= $3::numeric(20, 0)
        ORDER BY event_index DESC
        LIMIT $4::numeric(20, 0)
    "
    };

    let history_info = db_helpers::select_retry_or_panic::<super::models::CoinHistoryInfo>(
        balances_pool,
        query,
        &[
            account_id.to_string(),
            pagination.after_event_index.unwrap_or(0).to_string(),
            block.timestamp.to_string(),
            pagination.limit.to_string(),
        ],
    )
    .await?;

    let mut result: Vec<coin::schemas::HistoryItem> = vec![];
    for history in history_info {
        let involved_account_id: Option<types::AccountId> =
            if let Some(account_id) = history.involved_account_id {
                Some(near_primitives::types::AccountId::from_str(&account_id)?.into())
            } else {
                None
            };

        result.push(coin::schemas::HistoryItem {
            event_index: types::numeric::to_u128(&history.event_index)?.into(),
            involved_account_id,
            delta_balance: history.delta_balance.to_string(),
            balance: types::numeric::to_u128(&history.balance)?.into(),
            cause: history.cause,
            status: history.status,
            coin_metadata: super::get_near_metadata(),
            block_timestamp_nanos: types::numeric::to_u64(&history.block_timestamp_nanos)?.into(),
            block_height: types::numeric::to_u64(&history.block_height)?.into(),
        });
    }
    Ok(result)
}

pub(crate) async fn get_coin_history(
    pool: &sqlx::Pool<sqlx::Postgres>,
    pool_events: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    block: &db_helpers::Block,
    pagination: &types::query_params::Pagination,
) -> crate::Result<Vec<coin::schemas::HistoryItem>> {
    let balance_u128 = super::balance::get_ft_balance_by_contract(
        rpc_client,
        contract_id.clone(),
        account_id.clone(),
        block.height,
    )
    .await?;
    let mut current_balance = BigDecimal::from_str(&balance_u128.to_string()).map_err(|e| {
        errors::ErrorKind::InternalError(format!("Failed to parse BigDecimal from u128: {}", e))
    })?;

    let metadata = coin::schemas::CoinMetadata::from(
        super::metadata::get_ft_contract_metadata(rpc_client, contract_id.clone(), block.height)
            .await?,
    );

    // We don't have absolute_value in the DB.
    // We ask RPC for the first absolute_value, fill the in-between values with deltas, check by asking RPC for the last absolute_value and comparing the results.
    // If it matches, everything is good. If not, we return the error.

    // Problem: we can ask RPC only for the balance at the end of the block, while we may have the lines starting and ending in the middle of the block.
    // That's why we need to take slightly more lines so that we can match it with the RPC, and then filter these lines.
    // Potential issue here: the DB may contain only the part of the most fresh block (we may be in the middle of the writing process)
    // It can be easily solved by ignoring the most fresh block, but it will increase the lag between the response and the current blockchain state.
    // Since we anyway use transactional DB which guarantees that write process goes atomically, I don't want to do anything with that.
    // But, if we meet such issues in production, we may consider cutting the latest block.
    let query = if pagination.after_event_index.is_some() {
        r"
        WITH original_query as (
            SELECT
                event_index,
                involved_account_id,
                delta_amount,
                cause,
                status,
                block_timestamp,
                block_height
            FROM coin_events
            WHERE contract_account_id = $1
                AND affected_account_id = $2
                AND event_index < $3::numeric(38, 0)
            ORDER BY event_index desc
            LIMIT $5::numeric(20, 0)
        ), timestamps as (
            SELECT
                min(block_timestamp) min_block_timestamp,
                max(block_timestamp) max_block_timestamp
            FROM original_query
        )
        SELECT
            event_index,
            involved_account_id,
            delta_amount delta_balance,
            0::numeric(40, 0) balance, -- fill it later
            cause,
            status,
            block_timestamp block_timestamp_nanos,
            block_height
        FROM coin_events, timestamps
        WHERE contract_account_id = $1
            AND affected_account_id = $2
            AND block_timestamp >= min_block_timestamp
            AND block_timestamp <= max_block_timestamp
        ORDER BY event_index desc
    "
    } else {
        r"
        WITH original_query as (
            SELECT
                event_index,
                involved_account_id,
                delta_amount,
                cause,
                status,
                block_timestamp,
                block_height
            FROM coin_events
            WHERE contract_account_id = $1
                AND affected_account_id = $2
                AND block_timestamp <= $4::numeric(20, 0)
            ORDER BY event_index desc
            LIMIT $5::numeric(20, 0)
        ), timestamps as (
            SELECT
                min(block_timestamp) min_block_timestamp,
                max(block_timestamp) max_block_timestamp
            FROM original_query
        )
        SELECT
            event_index,
            involved_account_id,
            delta_amount delta_balance,
            0::numeric(40, 0) balance, -- fill it later
            cause,
            status,
            block_timestamp block_timestamp_nanos,
            block_height
        FROM coin_events, timestamps
        WHERE contract_account_id = $1
            AND affected_account_id = $2
            AND block_timestamp >= min_block_timestamp
            AND block_timestamp <= max_block_timestamp
        ORDER BY event_index desc
    "
    };
    let history =
        db_helpers::select_retry_or_panic::<coin::data_provider::models::CoinHistoryInfo>(
            pool_events,
            query,
            &[
                contract_id.to_string(),
                account_id.to_string(),
                pagination.after_event_index.unwrap_or(0).to_string(),
                block.timestamp.to_string(),
                pagination.limit.to_string(),
            ],
        )
        .await?;

    let mut result: Vec<coin::schemas::HistoryItem> = vec![];
    for db_info in history {
        let balance = current_balance.clone();

        if db_info.status == "SUCCESS" {
            current_balance -= db_info.delta_balance.clone();
            if current_balance.is_negative() {
                return Err(errors::ErrorKind::InternalError(format!(
                    "History is not supported for account {}. Contract {} provides inconsistent events which lead to negative balance",
                    account_id, contract_id
                ))
                    .into());
            }
        }

        let involved_account_id = match db_info.involved_account_id {
            Some(id) => Some(types::AccountId::from_str(&id)?),
            None => None,
        };
        let event_index = types::numeric::to_u128(&db_info.event_index)?;

        // We collect slightly more lines that we were asked for, because we can make RPC calls only at the end of the block
        // First clause filters latest redundant lines, second clause filters earliest redundant lines
        if pagination
            .after_event_index
            .map_or_else(|| true, |index| index > event_index)
            && result.len() < pagination.limit as usize
        {
            result.push(coin::schemas::HistoryItem {
                event_index: event_index.into(),
                cause: db_info.cause.clone(),
                involved_account_id,
                delta_balance: db_info.delta_balance.to_string(),
                balance: types::numeric::to_u128(&balance)?.into(),
                coin_metadata: metadata.clone(),
                block_timestamp_nanos: types::numeric::to_u64(&db_info.block_timestamp_nanos)?
                    .into(),
                block_height: types::numeric::to_u64(&db_info.block_height)?.into(),
                status: db_info.status,
            });
        }
    }

    let prev_block = if let Some(item) = result.last() {
        db_helpers::get_previous_block(pool, item.block_timestamp_nanos.0).await?
    } else {
        return Ok(result);
    };
    let earliest_balance = match super::balance::get_ft_balance_by_contract(
        rpc_client,
        contract_id.clone(),
        account_id.clone(),
        prev_block.height,
    )
    .await
    {
        Ok(x) => x,
        Err(e) => {
            if e.message.contains("does not exist at block_height") {
                0
            } else {
                return Err(e);
            }
        }
    };
    if types::numeric::to_u128(&current_balance)? != earliest_balance {
        return Err(errors::ErrorKind::InternalError(format!(
            "History is not supported for account {}. Contract {} provides inconsistent events",
            account_id, contract_id
        ))
        .into());
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;

    // TODO to support near history tests, we need to recollect the data
    // #[tokio::test]
    // async fn test_near_history() {
    //     let block = get_block();
    //     let pool = init_balances_db().await;
    //     let account = near_primitives::types::AccountId::from_str("vasya.near").unwrap();
    //     let pagination = types::query_params::Pagination {
    //         limit: 3,
    //         after_event_index: None,
    //     };
    //
    //     let history = get_near_history(&pool, &account, &block, &pagination).await;
    //     insta::assert_debug_snapshot!(history);
    // }
    //
    // #[tokio::test]
    // async fn test_near_history_next_page() {
    //     let block = get_block();
    //     let pool = init_balances_db().await;
    //     let account = near_primitives::types::AccountId::from_str("vasya.near").unwrap();
    //     let pagination = types::query_params::Pagination {
    //         limit: 3,
    //         after_event_index: Some(16197887992398226000000000000000003),
    //     };
    //
    //     let history = get_near_history(&pool, &account, &block, &pagination).await;
    //     insta::assert_debug_snapshot!(history);
    // }
    //
    // #[tokio::test]
    // async fn test_near_history_with_failed_receipts() {
    //     let block = db_helpers::Block {
    //         timestamp: 1618591017607373869,
    //         height: 34943083,
    //     };
    //     let pool = init_balances_db().await;
    //     let account = near_primitives::types::AccountId::from_str("zubkowi.near").unwrap();
    //     let pagination = types::query_params::Pagination {
    //         limit: 10,
    //         after_event_index: None,
    //     };
    //
    //     let history = get_near_history(&pool, &account, &block, &pagination).await;
    //     insta::assert_debug_snapshot!(history);
    // }
    //
    // #[tokio::test]
    // async fn test_near_history_account_does_not_exist() {
    //     let block = get_block();
    //     let pool = init_balances_db().await;
    //     let account =
    //         near_primitives::types::AccountId::from_str("two-idiots-and-a-half.near").unwrap();
    //     let pagination = types::query_params::Pagination {
    //         limit: 10,
    //         after_event_index: None,
    //     };
    //
    //     let history = get_near_history(&pool, &account, &block, &pagination)
    //         .await
    //         .unwrap();
    //     assert!(history.is_empty());
    // }

    #[tokio::test]
    async fn test_coin_history() {
        let pool = init_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("wrap.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("caoducanh98.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 10,
            after_event_index: None,
        };

        let history = get_coin_history(
            &pool,
            &pool_balances,
            &rpc_client,
            &contract,
            &account,
            &block,
            &pagination,
        )
        .await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_coin_history_next_page() {
        let pool = init_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let contract = near_primitives::types::AccountId::from_str("wrap.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("caoducanh98.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: Some(16351410322240000000000000000080003),
        };
        let block = db_helpers::checked_get_block_from_pagination(&pool, &pagination)
            .await
            .unwrap();

        let history = get_coin_history(
            &pool,
            &pool_balances,
            &rpc_client,
            &contract,
            &account,
            &block,
            &pagination,
        )
        .await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_coin_history_with_failed_receipts() {
        let pool = init_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("wrap.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("ulvend.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 10,
            after_event_index: None,
        };

        let history = get_coin_history(
            &pool,
            &pool_balances,
            &rpc_client,
            &contract,
            &account,
            &block,
            &pagination,
        )
        .await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_coin_history_account_does_not_exist() {
        let pool = init_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("wrap.near").unwrap();
        let account =
            near_primitives::types::AccountId::from_str("two-idiots-and-a-half.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 10,
            after_event_index: None,
        };

        let history = get_coin_history(
            &pool,
            &pool_balances,
            &rpc_client,
            &contract,
            &account,
            &block,
            &pagination,
        )
        .await
        .unwrap();
        assert!(history.is_empty());
    }

    #[tokio::test]
    async fn test_coin_history_contract_does_not_exist() {
        let pool = init_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let contract =
            near_primitives::types::AccountId::from_str("two-idiots-and-a-half.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 10,
            after_event_index: None,
        };

        let balance = get_coin_history(
            &pool,
            &pool_balances,
            &rpc_client,
            &contract,
            &account,
            &block,
            &pagination,
        )
        .await;
        insta::assert_debug_snapshot!(balance);
    }

    // TODO we need to rerun this test and update snapshot when the data will be collected
    // #[tokio::test]
    // async fn test_coin_history_contract_is_inconsistent() {
    //     let pool = init_db().await;
    //     let pool_events = init_events_db().await; let rpc_client = init_rpc();
    //     let block = get_block();
    //     let contract = near_primitives::types::AccountId::from_str("coin.asac.near").unwrap();
    //     let account = near_primitives::types::AccountId::from_str("olga.near").unwrap();
    //     let pagination = types::query_params::Pagination {
    //         limit: 10,
    //         after_event_index: None,
    //     };
    //
    //     let balance =
    //         get_coin_history(&pool, &pool_events, &rpc_client, &contract, &account, &block, &pagination).await;
    //     insta::assert_debug_snapshot!(balance);
    // }
}
