use crate::modules::ft;
use crate::{db_helpers, errors, types};
use num_traits::{Signed, ToPrimitive};
use sqlx::types::BigDecimal;
use std::str::FromStr;

pub(crate) async fn get_ft_history(
    db_helpers::ExplorerPool(pool_explorer): &db_helpers::ExplorerPool,
    db_helpers::BalancesPool(pool_balances): &db_helpers::BalancesPool,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    block: &db_helpers::Block,
    pagination: &types::query_params::Pagination,
) -> crate::Result<Vec<ft::schemas::HistoryItem>> {
    let metadata = ft::schemas::Metadata::from(
        super::metadata::get_ft_metadata(rpc_client, contract_id.clone(), block.height).await?,
    );

    let after_event_index = if let Some(index) = pagination.after_event_index {
        index
    } else {
        // +1 because we need to include given timestamp to result. Query has strict less operator
        db_helpers::timestamp_to_event_index(block.timestamp + 1)
    };

    // We don't have absolute_value in the DB.
    // We ask RPC for the first absolute_value, fill the in-between values with deltas, check by asking RPC for the last absolute_value and comparing the results.
    // If it matches, everything is good. If not, we return the error.

    // Problem: we can ask RPC only for the balance at the end of the block, while we may have the lines starting and ending in the middle of the block.
    // That's why we need to take slightly more lines so that we can match it with the RPC, and then filter these lines.
    // Potential issue here: the DB may contain only the part of the most fresh block (we may be in the middle of the writing process)
    // It can be easily solved by ignoring the most fresh block, but it will increase the lag between the response and the current blockchain state.
    // Since we anyway use transactional DB which guarantees that write process goes atomically, I don't want to do anything with that.
    // But, if we meet such issues in production, we may consider cutting the latest block.
    // TODO check the performance. We may add index on block_timestamp column, or we can hack and change block_timestamp to event_index
    let query = r"
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
                 AND event_index >= 16619903149406361800000000000000000 -- todo drop this when we finish collecting the data
             ORDER BY event_index desc
             LIMIT $4::numeric(20, 0)
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
     ";

    let history = db_helpers::select_retry_or_panic::<ft::data_provider::models::FtHistoryInfo>(
        pool_balances,
        query,
        &[
            contract_id.to_string(),
            account_id.to_string(),
            after_event_index.to_string(),
            pagination.limit.to_string(),
        ],
    )
    .await?;

    let mut current_balance = if let Some(first_item) = history.first() {
        let amount = super::balance::get_ft_amount(
            rpc_client,
            contract_id.clone(),
            account_id.clone(),
            first_item.block_height.to_u64().ok_or_else(|| {
                errors::ErrorKind::InternalError(
                    "Found negative block_height in coin_events table".to_string(),
                )
            })?,
        )
        .await?;
        BigDecimal::from_str(&amount.to_string()).map_err(|e| {
            errors::ErrorKind::InternalError(format!("Failed to parse BigDecimal from u128: {}", e))
        })?
    } else {
        return Ok(vec![]);
    };

    let mut result: Vec<ft::schemas::HistoryItem> = vec![];
    for db_info in history {
        let balance = current_balance.clone();

        if db_info.status == "SUCCESS" {
            current_balance -= db_info.delta_balance.clone();
            if current_balance.is_negative() {
                tracing::warn!(
                    target: crate::LOGGER_MSG,
                    "get_ft_history: found inconsistent events for contract {}, account {}, block {:#?}, pagination {:#?}",
                    contract_id,
                    account_id,
                    block,
                    pagination,
                );
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
            result.push(ft::schemas::HistoryItem {
                event_index: event_index.into(),
                cause: db_info.cause.clone(),
                involved_account_id,
                delta_balance: db_info.delta_balance.to_string(),
                balance: types::numeric::to_u128(&balance)?.into(),
                block_timestamp_nanos: types::numeric::to_u64(&db_info.block_timestamp_nanos)?
                    .into(),
                block_height: types::numeric::to_u64(&db_info.block_height)?.into(),
                status: db_info.status,
                metadata: metadata.clone(),
            });
        }
    }

    let prev_block = if let Some(item) = result.last() {
        db_helpers::get_previous_block(pool_explorer, item.block_timestamp_nanos.0).await?
    } else {
        return Ok(result);
    };
    let earliest_balance = match super::balance::get_ft_amount(
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
        tracing::warn!(
            target: crate::LOGGER_MSG,
            "get_ft_history: found inconsistent events for contract {}, account {}, block {:#?}, pagination {:#?}",
            contract_id,
            account_id,
            block,
            pagination,
        );
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
    use std::str::FromStr;

    #[tokio::test]
    async fn test_ft_history() {
        let pool_explorer = init_explorer_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str(
            "aaaaaa20d9e0e2461697782ef11675f668207961.factory.bridge.near",
        )
        .unwrap();
        let account = near_primitives::types::AccountId::from_str("aurora").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: None,
        };

        let balance = get_ft_history(
            &pool_explorer,
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

    #[tokio::test]
    async fn test_ft_history_next_page() {
        let pool_explorer = init_explorer_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let contract = near_primitives::types::AccountId::from_str(
            "aaaaaa20d9e0e2461697782ef11675f668207961.factory.bridge.near",
        )
        .unwrap();
        let account = near_primitives::types::AccountId::from_str("aurora").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: Some(16708552830626965310000000004000001),
        };
        let block = db_helpers::get_block_from_pagination(&pool_explorer, &pagination)
            .await
            .unwrap();

        let history = get_ft_history(
            &pool_explorer,
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
    async fn test_ft_history_next_page_in_the_middle_of_the_block() {
        let pool_explorer = init_explorer_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let contract = near_primitives::types::AccountId::from_str(
            "aaaaaa20d9e0e2461697782ef11675f668207961.factory.bridge.near",
        )
        .unwrap();
        let account = near_primitives::types::AccountId::from_str("aurora").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: Some(16704039164216566310000000004000001),
        };
        let block = db_helpers::get_block_from_pagination(&pool_explorer, &pagination)
            .await
            .unwrap();

        let history1 = get_ft_history(
            &pool_explorer,
            &pool_balances,
            &rpc_client,
            &contract,
            &account,
            &block,
            &pagination,
        )
        .await
        .unwrap();

        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: Some(history1.last().unwrap().event_index.0),
        };
        let block = db_helpers::get_block_from_pagination(&pool_explorer, &pagination)
            .await
            .unwrap();
        let history2 = get_ft_history(
            &pool_explorer,
            &pool_balances,
            &rpc_client,
            &contract,
            &account,
            &block,
            &pagination,
        )
        .await
        .unwrap();

        assert!(
            history1.last().unwrap().event_index > history2.first().unwrap().event_index,
            "Next page should not include event from previous page"
        );
        assert_eq!(
            history1.last().unwrap().block_height,
            history2.first().unwrap().block_height,
            "Block split in the middle expected"
        );
    }

    #[tokio::test]
    async fn test_ft_history_with_failed_receipts() {
        let pool_explorer = init_explorer_db().await;
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str(
            "52a047ee205701895ee06a375492490ec9c597ce.factory.bridge.near",
        )
        .unwrap();
        let account = near_primitives::types::AccountId::from_str("v2.ref-finance.near").unwrap();
        let pagination = types::query_params::Pagination {
            limit: 5,
            after_event_index: None,
        };

        let balance = get_ft_history(
            &pool_explorer,
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

    #[tokio::test]
    async fn test_ft_history_account_never_existed() {
        let pool_explorer = init_explorer_db().await;
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

        let history = get_ft_history(
            &pool_explorer,
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
    async fn test_ft_history_account_deleted() {
        let pool_explorer = init_explorer_db().await;
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

        let history = get_ft_history(
            &pool_explorer,
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
    async fn test_ft_history_contract_does_not_exist() {
        let pool_explorer = init_explorer_db().await;
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

        let balance = get_ft_history(
            &pool_explorer,
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

    // TODO write this test again when we finish collecting the data
    // I can't catch such cases on partially filled DB
    // #[tokio::test]
    // async fn test_ft_history_contract_is_inconsistent() {
    //     let pool_explorer = init_db().await;
    //     let pool_balances = init_balances_db().await;
    //     let rpc_client = init_rpc();
    //
    //     let contract = near_primitives::types::AccountId::from_str("tezeract.near").unwrap();
    //     let account = near_primitives::types::AccountId::from_str("puffball.near").unwrap();
    //     let pagination = types::query_params::Pagination {
    //         limit: 5,
    //         after_event_index: Some(16629459979548196140000003001000001),
    //     };
    //     let block = db_helpers::checked_get_block_from_pagination(&pool_explorer, &pagination)
    //         .await
    //         .unwrap();
    //
    //     let balance = get_ft_history(
    //         &pool_explorer,
    //         &pool_balances,
    //         &rpc_client,
    //         &contract,
    //         &account,
    //         &block,
    //         &pagination,
    //     )
    //     .await;
    //     insta::assert_debug_snapshot!(balance);
    // }
}
