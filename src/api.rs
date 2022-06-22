use std::str::FromStr;

use crate::{api_models, db_models, errors, rpc_calls, types, utils};

const RETRY_COUNT: usize = 10;

pub(crate) async fn native_balance(
    pool: &sqlx::Pool<sqlx::Postgres>,
    block: &types::Block,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<Vec<api_models::CoinInfo>> {
    let balances =
        utils::select_retry_or_panic::<db_models::AccountChangesBalance>(
            pool,
            r"WITH t AS (
                    SELECT affected_account_nonstaked_balance nonstaked, affected_account_staked_balance staked
                    FROM account_changes
                    WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
                    ORDER BY changed_in_block_timestamp DESC
                  )
                  SELECT * FROM t LIMIT 1
                 ",
            &[account_id.to_string(), block.timestamp.to_string()],
            RETRY_COUNT,
        ).await?;

    match balances.first() {
        Some(balance) => {
            // TODO support nonstaked, staked amounts
            let amount = utils::to_u128(&balance.nonstaked)? + utils::to_u128(&balance.staked)?;
            Ok(vec![api_models::CoinInfo {
                standard: "nearprotocol".to_string(),
                contract_account_id: None,
                amount: amount.into(),
                symbol: "NEAR".to_string(),
                decimals: 24,
                icon: None, // todo is the a right link to NEAR icon?
            }])
        }
        None => Err(errors::ErrorKind::DBError(format!(
            "Could not find the data in account_changes table for account_id {}",
            account_id
        ))
        .into()),
    }
}

pub(crate) async fn ft_balance(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<Vec<api_models::CoinInfo>> {
    let contracts = utils::select_retry_or_panic::<db_models::AccountId>(
        pool,
        r"SELECT DISTINCT emitted_by_contract_account_id account_id
              FROM assets__fungible_token_events
              WHERE token_old_owner_account_id = $1 OR token_new_owner_account_id = $1
             ",
        &[account_id.to_string(), block.timestamp.to_string()],
        RETRY_COUNT,
    )
    .await?;

    let mut balances: Vec<api_models::CoinInfo> = vec![];
    for contract in contracts {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&contract.account_id) {
            balances
                .push(ft_balance_for_contract(rpc_client, block, &contract_id, account_id).await?);
        }
    }
    Ok(balances)
}

pub(crate) async fn ft_balance_for_contract(
    // pool: &sqlx::Pool<sqlx::Postgres>, // hopefully we will take the data from DB in one day
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<api_models::CoinInfo> {
    let (balance, metadata) = (
        rpc_calls::get_ft_balance(
            rpc_client,
            contract_id.clone(),
            account_id.clone(),
            block.height,
        )
        .await?,
        rpc_calls::get_ft_metadata(rpc_client, contract_id.clone(), block.height).await?,
    );

    Ok(api_models::CoinInfo {
        standard: "nep141".to_string(),
        contract_account_id: Some(contract_id.to_string()),
        amount: balance.into(),
        symbol: metadata.symbol,
        decimals: metadata.decimals,
        icon: metadata.icon,
    })
}

pub(crate) async fn account_exists(
    pool: &sqlx::Pool<sqlx::Postgres>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> api_models::Result<bool> {
    // for the given timestamp, account exists if
    // 1. we have at least 1 row at action_receipt_actions table
    // 2. last successful action_kind != DELETE_ACCOUNT
    // TODO we are loosing +1 second here, it's painful
    Ok(utils::select_retry_or_panic::<db_models::ActionKind>(
        pool,
        r"SELECT action_kind::text
          FROM action_receipt_actions JOIN execution_outcomes ON action_receipt_actions.receipt_id = execution_outcomes.receipt_id
          WHERE receipt_predecessor_account_id = $1
              AND action_receipt_actions.receipt_included_in_block_timestamp <= $2::numeric(20, 0)
              AND execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID')
          ORDER BY receipt_included_in_block_timestamp DESC, index_in_action_receipt DESC
          LIMIT 1",
        &[account_id.to_string(), block_timestamp.to_string()],
        RETRY_COUNT,
    )
        .await?
        .first()
        .map(|kind| kind.action_kind != "DELETE_ACCOUNT")
        .unwrap_or_else(|| false))
}
