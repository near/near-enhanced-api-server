use crate::modules::ft;
use crate::{db_helpers, rpc_helpers, types};
use std::str::FromStr;

// TODO PHASE 2 pagination (recently updated go first), by artificial index added to assets__fungible_token_events
pub(crate) async fn get_ft_balances(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &db_helpers::Block,
    account_id: &near_primitives::types::AccountId,
    pagination: &types::query_params::Pagination,
) -> crate::Result<Vec<ft::schemas::FtBalance>> {
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

    let mut balances: Vec<ft::schemas::FtBalance> = vec![];
    for contract in contracts {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&contract.account_id) {
            balances.push(
                get_ft_balance_by_contract(rpc_client, block, &contract_id, account_id).await?,
            );
        }
    }
    Ok(balances)
}

// TODO PHASE 2 change RPC call to DB call by adding absolute amount values to assets__fungible_token_events
// TODO PHASE 2 add metadata tables to the DB, with periodic autoupdate
pub(crate) async fn get_ft_balance_by_contract(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &db_helpers::Block,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> crate::Result<ft::schemas::FtBalance> {
    let (amount, metadata) = (
        get_ft_amount(
            rpc_client,
            contract_id.clone(),
            account_id.clone(),
            block.height,
        )
        .await?,
        super::metadata::get_ft_metadata(rpc_client, contract_id.clone(), block.height).await?,
    );

    Ok(ft::schemas::FtBalance {
        amount: amount.into(),
        contract_account_id: contract_id.clone().into(),
        metadata: ft::schemas::Metadata {
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            decimals: metadata.decimals,
        },
    })
}

pub(crate) async fn get_ft_amount(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_coin_balances() {
        let pool = init_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();
        let pagination = types::query_params::Pagination { limit: 10 };
        let balance = get_ft_balances(&pool, &rpc_client, &block, &account, &pagination).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances_empty() {
        let pool = init_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let pagination = types::query_params::Pagination { limit: 10 };
        let balance = get_ft_balances(&pool, &rpc_client, &block, &account, &pagination)
            .await
            .unwrap();
        assert!(balance.is_empty());
    }

    #[tokio::test]
    async fn test_coin_balances_by_contract() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("nexp.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances_by_contract_no_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances_by_contract_other_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }
}