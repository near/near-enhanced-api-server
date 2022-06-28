use num_traits::ToPrimitive;
use std::str::FromStr;

use crate::{api_models, db_models, errors, rpc_calls, types, utils};

const RETRY_COUNT: usize = 1;

pub(crate) async fn native_balance(
    pool: &sqlx::Pool<sqlx::Postgres>,
    block: &types::Block,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<api_models::NearBalanceResponse> {
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
            let available = utils::to_u128(&balance.nonstaked)?;
            let staked = utils::to_u128(&balance.staked)?;
            Ok(api_models::NearBalanceResponse {
                total_balance: (available + staked).into(),
                available_balance: available.into(),
                staked_balance: staked.into(),
                metadata: api_models::CoinMetadata {
                    name: "NEAR blockchain native token".to_string(),
                    symbol: "NEAR".to_string(),
                    icon: None, // todo
                    decimals: 24,
                },
                block_timestamp_nanos: block.timestamp.into(),
                block_height: block.height.into(),
            })
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
    pagination: &types::Pagination,
) -> api_models::Result<Vec<api_models::Coin>> {
    let contracts = utils::select_retry_or_panic::<db_models::AccountId>(
        pool,
        r"WITH
            accounts AS (
              SELECT DISTINCT emitted_by_contract_account_id account_id
              FROM assets__fungible_token_events
              WHERE (token_old_owner_account_id = $1 OR token_new_owner_account_id = $1)
                  AND emitted_at_block_timestamp <= $2::numeric(20, 0)
              ORDER BY emitted_by_contract_account_id
              ),
            t AS (
              SELECT row_number() OVER (ORDER BY account_id) - 1 rownumber, account_id
              FROM accounts
              )
            SELECT account_id
            FROM t
            WHERE rownumber >= $3::numeric(20, 0) AND rownumber < $4::numeric(20, 0);
         ",
        &[
            account_id.to_string(),
            block.timestamp.to_string(),
            pagination.offset.to_string(),
            (pagination.offset + pagination.limit).to_string(),
        ],
        RETRY_COUNT,
    )
    .await?;

    let mut balances: Vec<api_models::Coin> = vec![];
    for contract in contracts {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&contract.account_id) {
            if let Some(ft) =
                ft_balance_for_contract(rpc_client, block, &contract_id, account_id).await?
            {
                balances.push(ft);
            }
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
) -> api_models::Result<Option<api_models::Coin>> {
    // todo test on contract that does not implement nep141
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

    Ok(Some(api_models::Coin {
        standard: "nep141".to_string(),
        contract_account_id: Some(contract_id.clone().into()),
        balance: balance.into(),
        metadata: api_models::CoinMetadata {
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            decimals: metadata.decimals,
        },
    }))
}

pub(crate) async fn coin_history(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<Vec<api_models::HistoryInfo>> {
    let mut last_balance = rpc_calls::get_ft_balance(
        rpc_client,
        contract_id.clone(),
        account_id.clone(),
        block.height,
    )
    .await?;

    let metadata: api_models::CoinMetadata =
        rpc_calls::get_ft_metadata(rpc_client, contract_id.clone(), block.height)
            .await?
            .into();

    // we collect the data from DB in straight order, then iter by rev order
    // the final result goes from latest to the earliest data
    let account_id = account_id.to_string();
    // todo here will be mts via union all
    // todo add enumeration artificial column. Think about MT here
    let ft_history_info = utils::select_retry_or_panic::<db_models::FtHistoryInfo>(
        pool,
        r"SELECT blocks.block_height,
                 blocks.block_timestamp,
                 assets__fungible_token_events.amount,
                 assets__fungible_token_events.event_kind::text,
                 assets__fungible_token_events.token_old_owner_account_id old_owner_id,
                 assets__fungible_token_events.token_new_owner_account_id new_owner_id
          FROM assets__fungible_token_events JOIN blocks
              ON assets__fungible_token_events.emitted_at_block_timestamp = blocks.block_timestamp
          WHERE emitted_by_contract_account_id = $1
              AND (token_old_owner_account_id = $2 OR token_new_owner_account_id = $2)
              AND emitted_at_block_timestamp <= $3::numeric(20, 0)
          ORDER BY emitted_at_block_timestamp
             ",
        &[
            contract_id.to_string(),
            account_id.clone(),
            block.timestamp.to_string(),
        ],
        RETRY_COUNT,
    )
    .await?;

    let mut result: Vec<api_models::HistoryInfo> = vec![];
    for db_info in ft_history_info.iter().rev() {
        let mut delta = utils::string_to_i128(&db_info.amount)?;
        let balance = last_balance;
        let involved_account_id = if account_id == db_info.old_owner_id {
            delta = -delta;
            if db_info.new_owner_id.is_empty() {
                None
            } else {
                Some(near_primitives::types::AccountId::from_str(
                    &db_info.new_owner_id,
                )?)
            }
        } else if account_id == db_info.new_owner_id {
            if db_info.old_owner_id.is_empty() {
                None
            } else {
                Some(near_primitives::types::AccountId::from_str(
                    &db_info.old_owner_id,
                )?)
            }
        } else {
            return Err(
                errors::ErrorKind::InternalError("todo unreachable code".to_string()).into(),
            );
        };

        if (last_balance as i128) - delta < 0 {
            return Err(errors::ErrorKind::InternalError(format!(
                "Balance could not be negative: account {}, contract {}",
                account_id, contract_id
            ))
            .into());
        }
        // todo rewrite this
        last_balance = ((last_balance as i128) - delta) as u128;

        result.push(api_models::HistoryInfo {
            action_kind: db_info.event_kind.clone(),
            involved_account_id: involved_account_id.map(|id| id.into()),
            delta_balance: delta.into(),
            balance: balance.into(),
            metadata: metadata.clone(),
            block_timestamp_nanos: utils::to_u64(&db_info.block_timestamp)?.into(),
            block_height: utils::to_u64(&db_info.block_height)?.into(),
        });
    }
    if let Some(info) = result.last() {
        if info.balance.0 != (info.delta_balance.0 as u128) {
            return Err(errors::ErrorKind::InternalError(format!(
                "We have found the money from nowhere for account {}, contract {}",
                account_id, contract_id
            ))
            .into());
        }
    }
    Ok(result)
}

// todo do we want to recheck the count by rpc? at least sometimes
pub(crate) async fn nft_count(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<Vec<api_models::NftsByContractInfo>> {
    // todo do we want to show zeros here? as the tokens that we had at one time, but not now
    // now we don't show them
    let contracts = utils::select_retry_or_panic::<db_models::NftCount>(
        pool,
        r"SELECT emitted_by_contract_account_id contract_id, count(*) count
          FROM assets__non_fungible_token_events
          WHERE token_new_owner_account_id = $1
              AND emitted_at_block_timestamp <= $2::numeric(20, 0)
          GROUP BY emitted_by_contract_account_id
         ",
        &[account_id.to_string(), block.timestamp.to_string()],
        RETRY_COUNT,
    )
    .await?;

    let mut result: Vec<api_models::NftsByContractInfo> = vec![];
    for contract in contracts {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&contract.contract_id)
        {
            let nft_count = contract.count.to_u32().ok_or_else(|| {
                errors::ErrorKind::InternalError(format!("Failed to parse u32 {}", contract.count))
            })?;
            let metadata =
                rpc_calls::get_nft_general_metadata(rpc_client, contract_id.clone(), block.height)
                    .await?;
            result.push(api_models::NftsByContractInfo {
                contract_account_id: contract_id.into(),
                nft_count,
                metadata,
            });
        }
    }
    Ok(result)
}

pub(crate) async fn nft_by_contract(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<Vec<api_models::NonFungibleToken>> {
    let tokens = utils::select_retry_or_panic::<db_models::NftId>(
        pool,
        r"SELECT token_id
          FROM assets__non_fungible_token_events
          WHERE emitted_by_contract_account_id = $1
              AND token_new_owner_account_id = $2
              AND emitted_at_block_timestamp <= $3::numeric(20, 0)

         ",
        &[
            contract_id.to_string(),
            account_id.to_string(),
            block.timestamp.to_string(),
        ],
        RETRY_COUNT,
    )
    .await?;

    let mut result: Vec<api_models::NonFungibleToken> = vec![];
    for token in tokens {
        result.push(
            rpc_calls::get_nft_metadata(
                rpc_client,
                contract_id.clone(),
                token.token_id,
                block.height,
            )
            .await?,
        );
    }
    Ok(result)
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
