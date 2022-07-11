use std::str::FromStr;

use crate::{api_models, db_models, errors, rpc_api, types, utils};

const DB_RETRY_COUNT: usize = 1;

pub(crate) async fn get_near_balance(
    pool: &sqlx::Pool<sqlx::Postgres>,
    block: &types::Block,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<api_models::NearBalanceResponse> {
    let balances =
        utils::select_retry_or_panic::<db_models::AccountChangesBalance>(
            pool,
            r"
                WITH t AS (
                    SELECT affected_account_nonstaked_balance nonstaked, affected_account_staked_balance staked
                    FROM account_changes
                    WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
                    ORDER BY changed_in_block_timestamp DESC
                )
                SELECT * FROM t LIMIT 1
            ",
            &[account_id.to_string(), block.timestamp.to_string()],
            DB_RETRY_COUNT,
        ).await?;

    match balances.first() {
        Some(balance) => {
            let available = utils::to_u128(&balance.nonstaked)?;
            let staked = utils::to_u128(&balance.staked)?;
            Ok(api_models::NearBalanceResponse {
                total_balance: (available + staked).into(),
                available_balance: available.into(),
                staked_balance: staked.into(),
                near_metadata: get_near_metadata(),
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

pub(crate) fn get_near_metadata() -> api_models::Metadata {
    api_models::Metadata {
        name: "NEAR blockchain native token".to_string(),
        symbol: "NEAR".to_string(),
        // TODO PHASE 2 re-check the icon. It's the best I can find
        icon: Some("https://raw.githubusercontent.com/near/near-wallet/7ef3c824404282b76b36da2dff4f3e593e7f928d/packages/frontend/src/images/near.svg".to_string()),
        decimals: 24,
    }
}

// TODO PHASE 2 pagination (recently updated go first), by artificial index added to assets__fungible_token_events
pub(crate) async fn get_ft_balances(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    account_id: &near_primitives::types::AccountId,
    pagination: &types::CoinBalancesPagination,
) -> api_models::Result<Vec<api_models::Coin>> {
    let query = r"
        SELECT DISTINCT emitted_by_contract_account_id account_id
        FROM assets__fungible_token_events
        WHERE (token_old_owner_account_id = $1 OR token_new_owner_account_id = $1)
            AND emitted_at_block_timestamp <= $2::numeric(20, 0)
        ORDER BY emitted_by_contract_account_id
        LIMIT $3::numeric(20, 0)
    ";
    let contracts = utils::select_retry_or_panic::<db_models::AccountId>(
        pool,
        query,
        &[
            account_id.to_string(),
            block.timestamp.to_string(),
            pagination.limit.to_string(),
        ],
        DB_RETRY_COUNT,
    )
    .await?;

    let mut balances: Vec<api_models::Coin> = vec![];
    for contract in contracts {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&contract.account_id) {
            balances.push(
                get_ft_balance_for_contract(rpc_client, block, &contract_id, account_id).await?,
            );
        }
    }
    Ok(balances)
}

// TODO PHASE 2 change RPC call to DB call by adding absolute amount values to assets__fungible_token_events
// TODO PHASE 2 add metadata tables to the DB, with periodic autoupdate
pub(crate) async fn get_ft_balance_for_contract(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> api_models::Result<api_models::Coin> {
    let (balance, metadata) = (
        rpc_api::get_ft_balance(
            rpc_client,
            contract_id.clone(),
            account_id.clone(),
            block.height,
        )
        .await?,
        rpc_api::get_ft_contract_metadata(rpc_client, contract_id.clone(), block.height).await?,
    );

    Ok(api_models::Coin {
        standard: "nep141".to_string(),
        contract_account_id: Some(contract_id.clone().into()),
        balance: balance.into(),
        coin_metadata: api_models::Metadata {
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            decimals: metadata.decimals,
        },
    })
}

// TODO PHASE 2 pagination by artificial index added to balance_changes
pub(crate) async fn get_near_history(
    balances_pool: &sqlx::Pool<sqlx::Postgres>,
    account_id: &near_primitives::types::AccountId,
    pagination: &types::HistoryPagination,
) -> api_models::Result<Vec<api_models::NearHistoryItem>> {
    let query = r"
        SELECT
            involved_account_id,
            delta_nonstaked_amount + delta_staked_amount delta_balance,
            delta_nonstaked_amount delta_available_balance,
            delta_staked_amount delta_staked_balance,
            absolute_nonstaked_amount + absolute_staked_amount total_balance,
            absolute_nonstaked_amount available_balance,
            absolute_staked_amount staked_balance,
            cause,
            block_timestamp block_timestamp_nanos
        FROM balance_changes
        WHERE affected_account_id = $1 AND block_timestamp < $2::numeric(20, 0)
        ORDER BY block_timestamp DESC
        LIMIT $3::numeric(20, 0)
    ";

    let history_info = utils::select_retry_or_panic::<db_models::NearHistoryInfo>(
        balances_pool,
        query,
        &[
            account_id.to_string(),
            pagination.block_timestamp.to_string(),
            pagination.limit.to_string(),
        ],
        DB_RETRY_COUNT,
    )
    .await?;

    let mut result: Vec<api_models::NearHistoryItem> = vec![];
    for history in history_info {
        result.push(history.try_into()?);
    }
    Ok(result)
}

// TODO PHASE 2 pagination by artificial index added to assets__fungible_token_events
// TODO PHASE 2 change RPC call to DB call by adding absolute amount values to assets__fungible_token_events
// TODO PHASE 2 make the decision about separate FT/MT tables or one table. Pagination implementation depends on this
pub(crate) async fn get_ft_history(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
    pagination: &types::HistoryPagination,
) -> api_models::Result<Vec<api_models::CoinHistoryItem>> {
    let mut last_balance = rpc_api::get_ft_balance(
        rpc_client,
        contract_id.clone(),
        account_id.clone(),
        pagination.block_height,
    )
    .await?;

    let account_id = account_id.to_string();
    let query = r"
        SELECT blocks.block_height,
               blocks.block_timestamp,
               assets__fungible_token_events.amount::numeric(45, 0),
               assets__fungible_token_events.event_kind::text,
               assets__fungible_token_events.token_old_owner_account_id old_owner_id,
               assets__fungible_token_events.token_new_owner_account_id new_owner_id
        FROM assets__fungible_token_events
            JOIN blocks ON assets__fungible_token_events.emitted_at_block_timestamp = blocks.block_timestamp
            JOIN execution_outcomes ON assets__fungible_token_events.emitted_for_receipt_id = execution_outcomes.receipt_id
        WHERE emitted_by_contract_account_id = $1
            AND execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID')
            AND (token_old_owner_account_id = $2 OR token_new_owner_account_id = $2)
            AND emitted_at_block_timestamp <= $3::numeric(20, 0)
        ORDER BY emitted_at_block_timestamp desc
        LIMIT $4::numeric(20, 0)
    ";
    let ft_history_info = utils::select_retry_or_panic::<db_models::FtHistoryInfo>(
        pool,
        query,
        &[
            contract_id.to_string(),
            account_id.clone(),
            pagination.block_timestamp.to_string(),
            pagination.limit.to_string(),
        ],
        DB_RETRY_COUNT,
    )
    .await?;

    let mut result: Vec<api_models::CoinHistoryItem> = vec![];
    for db_info in ft_history_info {
        let mut delta: i128 = utils::to_i128(&db_info.amount)?;
        let balance = last_balance;
        // TODO PHASE 2 maybe we want to change assets__fungible_token_events also to affected/involved?
        let involved_account_id = if account_id == db_info.old_owner_id {
            delta = -delta;
            utils::extract_account_id(&db_info.new_owner_id)?
        } else if account_id == db_info.new_owner_id {
            utils::extract_account_id(&db_info.old_owner_id)?
        } else {
            return Err(
                errors::ErrorKind::InternalError(
                    format!("The account {} should be sender or receiver ({}, {}). If you see this, please create the issue",
                            account_id, db_info.old_owner_id, db_info.new_owner_id)).into(),
            );
        };

        // TODO PHASE 2 this strange error will go away after we add absolute amounts to the DB
        if (last_balance as i128) - delta < 0 {
            return Err(errors::ErrorKind::InternalError(format!(
                "Balance could not be negative: account {}, contract {}",
                account_id, contract_id
            ))
            .into());
        }
        last_balance = ((last_balance as i128) - delta) as u128;

        result.push(api_models::CoinHistoryItem {
            action_kind: db_info.event_kind.clone(),
            involved_account_id: involved_account_id.map(|id| id.into()),
            delta_balance: delta.into(),
            balance: balance.into(),
            coin_metadata: None,
            block_timestamp_nanos: utils::to_u64(&db_info.block_timestamp)?.into(),
            block_height: utils::to_u64(&db_info.block_height)?.into(),
        });
    }
    Ok(result)
}

// TODO PHASE 2 pagination by artificial index added to assets__non_fungible_token_events
pub(crate) async fn get_nft_count(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &types::Block,
    account_id: &near_primitives::types::AccountId,
    pagination: &api_models::PaginationParams,
) -> api_models::Result<Vec<api_models::NftCollectionByContract>> {
    let query = r"
        WITH relevant_events AS (
            SELECT emitted_at_block_timestamp, token_id, emitted_by_contract_account_id, token_old_owner_account_id, token_new_owner_account_id
            FROM assets__non_fungible_token_events
                JOIN execution_outcomes ON assets__non_fungible_token_events.emitted_for_receipt_id = execution_outcomes.receipt_id
            WHERE
                -- if it works slow, we need to create table daily_nft_count_by_contract_and_user, and this query will run only over the last day
                -- emitted_at_block_timestamp > start_of_day AND
                emitted_at_block_timestamp <= $2::numeric(20, 0)
                AND execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID')
                AND (token_new_owner_account_id = $1 OR token_old_owner_account_id = $1)
        ),
        outgoing_events_count AS (
            SELECT emitted_by_contract_account_id, count(*) * -1 cnt FROM relevant_events
            WHERE token_old_owner_account_id = $1
            GROUP BY emitted_by_contract_account_id
        ),
        ingoing_events_count AS (
            SELECT emitted_by_contract_account_id, count(*) cnt FROM relevant_events
            WHERE token_new_owner_account_id = $1
            GROUP BY emitted_by_contract_account_id
        ),
        counts AS (
            SELECT ingoing_events_count.emitted_by_contract_account_id,
                -- coalesce changes null to the given parameter
                coalesce(ingoing_events_count.cnt, 0) + coalesce(outgoing_events_count.cnt, 0) cnt
            FROM ingoing_events_count FULL JOIN outgoing_events_count
                ON ingoing_events_count.emitted_by_contract_account_id = outgoing_events_count.emitted_by_contract_account_id
        ),
        counts_with_timestamp AS (
            SELECT distinct ON (counts.emitted_by_contract_account_id) counts.emitted_by_contract_account_id contract_id,
                cnt count,
                emitted_at_block_timestamp last_updated_at_timestamp
            FROM counts JOIN relevant_events ON counts.emitted_by_contract_account_id = relevant_events.emitted_by_contract_account_id
            WHERE cnt > 0
            ORDER BY counts.emitted_by_contract_account_id, emitted_at_block_timestamp DESC
        )
        SELECT * FROM counts_with_timestamp
        -- WHERE last_updated_at_timestamp < $3::numeric(20, 0) -- phase 2 pagination will be covered here
        ORDER BY last_updated_at_timestamp DESC
        LIMIT $3::numeric(20, 0)
    ";

    let info_by_contract = utils::select_retry_or_panic::<db_models::NftCount>(
        pool,
        query,
        &[
            account_id.to_string(),
            block.timestamp.to_string(),
            pagination
                .limit
                .unwrap_or(crate::DEFAULT_PAGE_LIMIT)
                .to_string(),
        ],
        DB_RETRY_COUNT,
    )
    .await?;

    let mut result: Vec<api_models::NftCollectionByContract> = vec![];
    for info in info_by_contract {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&info.contract_id) {
            let metadata =
                rpc_api::get_nft_contract_metadata(rpc_client, contract_id.clone(), block.height)
                    .await
                    .unwrap_or_else(|_| get_default_nft_contract_metadata());
            result.push(api_models::NftCollectionByContract {
                contract_account_id: contract_id.into(),
                nft_count: info.count as u32,
                last_updated_at_timestamp_nanos: utils::to_u128(&info.last_updated_at_timestamp)?
                    .into(),
                contract_metadata: metadata,
            });
        }
    }
    Ok(result)
}

// Metadata is the required part of the standard.
// Unfortunately, some contracts (e.g. `nft.nearapps.near`) do not implement it.
// We should give at least anything for such contracts when we serve the overview information.
fn get_default_nft_contract_metadata() -> api_models::NftContractMetadata {
    api_models::NftContractMetadata {
        spec: "nft-1.0.0".to_string(),
        name: "The contract did not provide the metadata".to_string(),
        symbol: "The contract did not provide the symbol".to_string(),
        icon: None,
        base_uri: None,
        reference: None,
        reference_hash: None,
    }
}

// TODO PHASE 2 pagination by artificial index added to assets__non_fungible_token_events
pub(crate) async fn get_nft_history(
    pool: &sqlx::Pool<sqlx::Postgres>,
    contract_id: &near_primitives::types::AccountId,
    token_id: &str,
    pagination: &types::HistoryPagination,
) -> api_models::Result<Vec<api_models::NftHistoryItem>> {
    let query = r"
        SELECT event_kind::text action_kind,
               token_old_owner_account_id old_account_id,
               token_new_owner_account_id new_account_id,
               emitted_at_block_timestamp block_timestamp_nanos,
               block_height
        FROM assets__non_fungible_token_events
            JOIN blocks ON assets__non_fungible_token_events.emitted_at_block_timestamp = blocks.block_timestamp
            JOIN execution_outcomes ON assets__non_fungible_token_events.emitted_for_receipt_id = execution_outcomes.receipt_id
        WHERE token_id = $1
            AND emitted_by_contract_account_id = $2
            AND emitted_at_block_timestamp < $3::numeric(20, 0)
            AND execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID')
        ORDER BY emitted_at_block_timestamp DESC
        LIMIT $4::numeric(20, 0)
    ";
    let history_items = utils::select_retry_or_panic::<db_models::NftHistoryInfo>(
        pool,
        query,
        &[
            token_id.to_string(),
            contract_id.to_string(),
            pagination.block_timestamp.to_string(),
            pagination.limit.to_string(),
        ],
        DB_RETRY_COUNT,
    )
    .await?;

    let mut result: Vec<api_models::NftHistoryItem> = vec![];
    for history in history_items {
        result.push(history.try_into()?);
    }
    Ok(result)
}

// TODO PHASE 2+ we are loosing +1 second here, it's painful. It could be computed much easier in Aurora DB
pub(crate) async fn does_account_exist(
    pool: &sqlx::Pool<sqlx::Postgres>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> api_models::Result<bool> {
    // for the given timestamp, account exists if
    // 1. we have at least 1 row at action_receipt_actions table
    // 2. last successful action_kind != DELETE_ACCOUNT
    let query = r"
        SELECT action_kind::text
        FROM action_receipt_actions JOIN execution_outcomes ON action_receipt_actions.receipt_id = execution_outcomes.receipt_id
        WHERE receipt_predecessor_account_id = $1
            AND action_receipt_actions.receipt_included_in_block_timestamp <= $2::numeric(20, 0)
            AND execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID')
        ORDER BY receipt_included_in_block_timestamp DESC, index_in_action_receipt DESC
        LIMIT 1
     ";
    Ok(utils::select_retry_or_panic::<db_models::ActionKind>(
        pool,
        query,
        &[account_id.to_string(), block_timestamp.to_string()],
        DB_RETRY_COUNT,
    )
    .await?
    .first()
    .map(|kind| kind.action_kind != "DELETE_ACCOUNT")
    .unwrap_or_else(|| false))
}

pub(crate) async fn get_block_from_params(
    pool: &sqlx::Pool<sqlx::Postgres>,
    params: &api_models::BlockParams,
) -> api_models::Result<types::Block> {
    if let Some(block_height) = params.block_height {
        match utils::select_retry_or_panic::<db_models::Block>(
            pool,
            "SELECT block_height, block_timestamp FROM blocks WHERE block_height = $1::numeric(20, 0)",
            &[block_height.0.to_string()],
            DB_RETRY_COUNT,
        )
            .await?
            .first() {
            None => Err(errors::ErrorKind::DBError(format!("block_height {} is not found", block_height.0)).into()),
            Some(block) => Ok(types::Block::try_from(block)?)
        }
    } else if let Some(block_timestamp) = params.block_timestamp_nanos {
        match utils::select_retry_or_panic::<db_models::Block>(
            pool,
            r"SELECT block_height, block_timestamp
              FROM blocks
              WHERE block_timestamp <= $1::numeric(20, 0)
              ORDER BY block_timestamp DESC
              LIMIT 1",
            &[block_timestamp.0.to_string()],
            DB_RETRY_COUNT,
        )
        .await?
        .first()
        {
            None => get_first_block(pool).await,
            Some(block) => Ok(types::Block::try_from(block)?),
        }
    } else {
        get_last_block(pool).await
    }
}

async fn get_first_block(pool: &sqlx::Pool<sqlx::Postgres>) -> api_models::Result<types::Block> {
    match utils::select_retry_or_panic::<db_models::Block>(
        pool,
        r"SELECT block_height, block_timestamp
          FROM blocks
          ORDER BY block_timestamp
          LIMIT 1",
        &[],
        DB_RETRY_COUNT,
    )
    .await?
    .first()
    {
        None => Err(errors::ErrorKind::DBError("blocks table is empty".to_string()).into()),
        Some(block) => Ok(types::Block::try_from(block)?),
    }
}

pub(crate) async fn get_last_block(
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> api_models::Result<types::Block> {
    match utils::select_retry_or_panic::<db_models::Block>(
        pool,
        r"SELECT block_height, block_timestamp
          FROM blocks
          ORDER BY block_timestamp DESC
          LIMIT 1",
        &[],
        DB_RETRY_COUNT,
    )
    .await?
    .first()
    {
        None => Err(errors::ErrorKind::DBError("blocks table is empty".to_string()).into()),
        Some(block) => Ok(types::Block::try_from(block)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn init() -> (
        sqlx::Pool<sqlx::Postgres>,
        near_jsonrpc_client::JsonRpcClient,
        types::Block,
    ) {
        dotenv::dotenv().ok();
        let db_url = &std::env::var("DATABASE_URL").expect("failed to get database url");
        let rpc_url = &std::env::var("RPC_URL").expect("failed to get RPC url");
        let connector = near_jsonrpc_client::JsonRpcClient::new_client();

        (
            sqlx::PgPool::connect(db_url)
                .await
                .expect("failed to connect to the database"),
            connector.connect(rpc_url),
            types::Block {
                timestamp: 1655571176644255779,
                height: 68000000,
            },
        )
    }

    #[tokio::test]
    async fn test_near_balance() {
        let (pool, _, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("tomato.near").unwrap();
        let balance = get_near_balance(&pool, &block, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();
        let pagination = types::CoinBalancesPagination { limit: 10 };
        let balance = get_ft_balances(&pool, &rpc_client, &block, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance_no_fts() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let pagination = types::CoinBalancesPagination { limit: 10 };
        let balance = get_ft_balances(&pool, &rpc_client, &block, &account, &pagination)
            .await
            .unwrap();
        assert!(balance.is_empty());
    }

    #[tokio::test]
    async fn test_ft_balance_for_contract() {
        let (_, rpc_client, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("nexp.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_for_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance_for_contract_no_contract_deployed() {
        let (_, rpc_client, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_for_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance_for_contract_other_contract_deployed() {
        let (_, rpc_client, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_for_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_near_history() {
        let (_, _, block) = init().await;
        // Using the other pool because we have this table at the other DB
        let url_balances =
            &std::env::var("DATABASE_URL_BALANCES").expect("failed to get database url");
        let pool = sqlx::PgPool::connect(url_balances)
            .await
            .expect("failed to connect to the balances database");
        let account = near_primitives::types::AccountId::from_str("vasya.near").unwrap();
        let pagination = types::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let balance = get_near_history(&pool, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_history() {
        let (pool, rpc_client, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();
        let account = near_primitives::types::AccountId::from_str("pushxo.near").unwrap();
        let pagination = types::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let balance = get_ft_history(&pool, &rpc_client, &contract, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_nft_count() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("blondjesus.near").unwrap();
        let pagination = api_models::PaginationParams { limit: Some(10) };

        let nft_count = get_nft_count(&pool, &rpc_client, &block, &account, &pagination).await;
        insta::assert_debug_snapshot!(nft_count);
    }

    #[tokio::test]
    async fn test_nft_count_no_nfts() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("frol.near").unwrap();
        let pagination = api_models::PaginationParams { limit: None };

        let nft_count = get_nft_count(&pool, &rpc_client, &block, &account, &pagination)
            .await
            .unwrap();
        assert!(nft_count.is_empty());
    }

    #[tokio::test]
    async fn test_nft_count_with_contracts_with_no_metadata() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("vlad.near").unwrap();
        let pagination = api_models::PaginationParams { limit: Some(10) };

        let nft_count = get_nft_count(&pool, &rpc_client, &block, &account, &pagination).await;
        insta::assert_debug_snapshot!(nft_count);
    }

    #[tokio::test]
    async fn test_nft_count_with_no_failed_receipts_in_result() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("kbneoburner3.near").unwrap();
        let pagination = api_models::PaginationParams { limit: None };

        let nft_count = get_nft_count(&pool, &rpc_client, &block, &account, &pagination).await;
        insta::assert_debug_snapshot!(nft_count);
    }

    #[tokio::test]
    async fn test_nft_history() {
        let (pool, _, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "293708:1";
        let pagination = types::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let history = get_nft_history(&pool, &contract, token, &pagination).await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_nft_history_with_no_failed_receipts_in_result() {
        let (pool, _, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("thebullishbulls.near").unwrap();
        let token = "1349";
        let pagination = types::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let history = get_nft_history(&pool, &contract, token, &pagination).await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_nft_history_token_does_not_exist() {
        let (pool, _, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "no_such_token";
        let pagination = types::HistoryPagination {
            block_height: block.height,
            block_timestamp: block.timestamp,
            limit: 10,
        };

        let history = get_nft_history(&pool, &contract, token, &pagination)
            .await
            .unwrap();
        assert!(history.is_empty());
    }
}
