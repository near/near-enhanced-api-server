use crate::modules::ft;
use crate::{db_helpers, errors, types};

// TODO PHASE 2 pagination by artificial index added to assets__fungible_token_events
// TODO PHASE 2 change RPC call to DB call by adding absolute amount values to assets__fungible_token_events
// TODO PHASE 2 make the decision about separate FT/MT tables or one table. Pagination implementation depends on this
pub(crate) async fn get_ft_history(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    pagination: &types::query_params::HistoryPagination,
) -> crate::Result<Vec<ft::schemas::HistoryItem>> {
    // this is temp solution before we make changes to the DB
    let mut last_balance = super::balance::get_ft_amount(
        rpc_client,
        contract_id.clone(),
        account_id.clone(),
        pagination.block_height,
    )
    .await?;
    let metadata = ft::schemas::Metadata::from(
        super::metadata::get_ft_metadata(rpc_client, contract_id.clone(), pagination.block_height)
            .await?,
    );

    let account_id = account_id.to_string();
    let query = r"
        SELECT
            -- blocks.block_height,
            blocks.block_timestamp,
            assets__fungible_token_events.amount::numeric(45, 0),
            assets__fungible_token_events.event_kind::text cause,
            CASE WHEN execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID') THEN 'SUCCESS'
                ELSE 'FAILURE'
            END status,
            assets__fungible_token_events.token_old_owner_account_id old_owner_id,
            assets__fungible_token_events.token_new_owner_account_id new_owner_id
        FROM assets__fungible_token_events
            JOIN blocks ON assets__fungible_token_events.emitted_at_block_timestamp = blocks.block_timestamp
            JOIN execution_outcomes ON assets__fungible_token_events.emitted_for_receipt_id = execution_outcomes.receipt_id
        WHERE emitted_by_contract_account_id = $1
            AND (token_old_owner_account_id = $2 OR token_new_owner_account_id = $2)
            AND emitted_at_block_timestamp <= $3::numeric(20, 0)
        ORDER BY emitted_at_block_timestamp desc
        LIMIT $4::numeric(20, 0)
    ";
    let history_info = db_helpers::select_retry_or_panic::<super::models::FtHistoryInfo>(
        pool,
        query,
        &[
            contract_id.to_string(),
            account_id.clone(),
            pagination.block_timestamp.to_string(),
            pagination.limit.to_string(),
        ],
    )
    .await?;

    let mut result: Vec<ft::schemas::HistoryItem> = vec![];
    for db_info in history_info {
        let mut delta: i128 = types::numeric::to_i128(&db_info.amount)?;
        let balance = last_balance;
        // TODO PHASE 2 maybe we want to change assets__fungible_token_events also to affected/involved?
        let involved_account_id = if account_id == db_info.old_owner_id {
            delta = -delta;
            types::account_id::extract_account_id(&db_info.new_owner_id)?
        } else if account_id == db_info.new_owner_id {
            types::account_id::extract_account_id(&db_info.old_owner_id)?
        } else {
            return Err(
                errors::ErrorKind::InternalError(
                    format!("The account {} should be sender or receiver ({}, {}). If you see this, please create the issue",
                            account_id, db_info.old_owner_id, db_info.new_owner_id)).into(),
            );
        };

        if db_info.status == "SUCCESS" {
            // TODO PHASE 2 this strange error will go away after we add absolute amounts to the DB
            if (last_balance as i128) - delta < 0 {
                return Err(errors::ErrorKind::InternalError(format!(
                    "Balance could not be negative: account {}, contract {}",
                    account_id, contract_id
                ))
                .into());
            }
            last_balance = ((last_balance as i128) - delta) as u128;
        }

        result.push(ft::schemas::HistoryItem {
            cause: db_info.cause.clone(),
            involved_account_id: involved_account_id.map(|id| id.into()),
            delta_balance: delta.into(),
            balance: balance.into(),
            metadata: metadata.clone(),
            block_timestamp_nanos: types::numeric::to_u64(&db_info.block_timestamp)?.into(),
            // block_height: types::numeric::to_u64(&db_info.block_height)?.into(),
            status: db_info.status,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;

    #[tokio::test]
    async fn test_coin_history() {
        let pool = init_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();
        let account = near_primitives::types::AccountId::from_str("pushxo.near").unwrap();
        let pagination = types::query_params::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let balance = get_ft_history(&pool, &rpc_client, &contract, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_history_with_failed_receipts() {
        let pool = init_db().await;
        let rpc_client = init_rpc();
        let block = db_helpers::Block {
            timestamp: 1651062637353692535,
            height: 64408633,
        };
        let contract =
            near_primitives::types::AccountId::from_str("sweat_token_testing.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("intmainreturn0.near").unwrap();
        let pagination = types::query_params::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let balance = get_ft_history(&pool, &rpc_client, &contract, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }
}
