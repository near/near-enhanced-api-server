use crate::modules::coin;
use crate::{db_helpers, errors, rpc_helpers, types};
use std::str::FromStr;

pub(crate) async fn get_near_balance(
    pool: &sqlx::Pool<sqlx::Postgres>,
    block: &db_helpers::Block,
    account_id: &near_primitives::types::AccountId,
) -> crate::Result<coin::schemas::NearBalanceResponse> {
    let balances =
        db_helpers::select_retry_or_panic::<super::models::AccountChangesBalance>(
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
        ).await?;

    match balances.first() {
        Some(balance) => {
            let available = types::numeric::to_u128(&balance.nonstaked)?;
            let staked = types::numeric::to_u128(&balance.staked)?;
            Ok(coin::schemas::NearBalanceResponse {
                total_balance: (available + staked).into(),
                available_balance: available.into(),
                staked_balance: staked.into(),
                near_metadata: super::metadata::get_near_metadata(),
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

// todo coin naming
// TODO PHASE 2 pagination (recently updated go first), by artificial index added to assets__fungible_token_events
pub(crate) async fn get_ft_balances(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &db_helpers::Block,
    account_id: &near_primitives::types::AccountId,
    pagination: &types::query_params::CoinBalancesPagination,
) -> crate::Result<Vec<coin::schemas::Coin>> {
    let query = r"
        SELECT DISTINCT emitted_by_contract_account_id account_id
        FROM assets__fungible_token_events
        WHERE (token_old_owner_account_id = $1 OR token_new_owner_account_id = $1)
            AND emitted_at_block_timestamp <= $2::numeric(20, 0)
        ORDER BY emitted_by_contract_account_id
        LIMIT $3::numeric(20, 0)
    ";
    let contracts = db_helpers::select_retry_or_panic::<db_helpers::AccountId>(
        pool,
        query,
        &[
            account_id.to_string(),
            block.timestamp.to_string(),
            pagination.limit.to_string(),
        ],
    )
    .await?;

    let mut balances: Vec<coin::schemas::Coin> = vec![];
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
    block: &db_helpers::Block,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> crate::Result<coin::schemas::Coin> {
    let (balance, metadata) = (
        get_ft_balance(
            rpc_client,
            contract_id.clone(),
            account_id.clone(),
            block.height,
        )
        .await?,
        super::metadata::get_ft_contract_metadata(rpc_client, contract_id.clone(), block.height)
            .await?,
    );

    Ok(coin::schemas::Coin {
        standard: "nep141".to_string(),
        contract_account_id: Some(contract_id.clone().into()),
        balance: balance.into(),
        coin_metadata: coin::schemas::Metadata {
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            decimals: metadata.decimals,
        },
    })
}

pub(crate) async fn get_ft_balance(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
    block_height: u64,
) -> crate::Result<u128> {
    let request = rpc_helpers::get_function_call_request(
        block_height,
        contract_id.clone(),
        "ft_balance_of",
        serde_json::json!({ "account_id": account_id }),
    );
    let response =
        rpc_helpers::wrapped_call(rpc_client, request, block_height, &contract_id).await?;
    Ok(serde_json::from_slice::<types::U128>(&response.result)?.0)
}

impl From<coin::schemas::NearBalanceResponse> for coin::schemas::Coin {
    fn from(near_coin: coin::schemas::NearBalanceResponse) -> Self {
        coin::schemas::Coin {
            standard: "nearprotocol".to_string(),
            balance: near_coin.total_balance,
            contract_account_id: None,
            coin_metadata: near_coin.near_metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_near_balance() {
        let (pool, _, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("tomato.near").unwrap();
        let balance = get_near_balance(&pool, &block, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();
        let pagination = types::query_params::CoinBalancesPagination { limit: 10 };
        let balance = get_ft_balances(&pool, &rpc_client, &block, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances_no_fts() {
        let (pool, rpc_client, block) = init().await;
        let account = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let pagination = types::query_params::CoinBalancesPagination { limit: 10 };
        let balance = get_ft_balances(&pool, &rpc_client, &block, &account, &pagination)
            .await
            .unwrap();
        assert!(balance.is_empty());
    }

    #[tokio::test]
    async fn test_coin_balance_for_contract() {
        let (_, rpc_client, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("nexp.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_for_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance_todo_compare_with_prev() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();
        let account = near_primitives::types::AccountId::from_str("cgarls.near").unwrap();

        let balance = get_ft_balance(&rpc_client, contract, account, block_height)
            .await
            .unwrap();
        assert_eq!(17201878399999996928, balance);
    }

    #[tokio::test]
    async fn test_coin_balance_for_contract_no_contract_deployed() {
        let (_, rpc_client, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_for_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balance_for_contract_other_contract_deployed() {
        let (_, rpc_client, block) = init().await;
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_for_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }
}
