use crate::modules::ft;
use crate::{db_helpers, rpc_helpers, types};
use std::str::FromStr;

pub(crate) async fn get_ft_balances(
    db_helpers::BalancesPool(pool_balances): &db_helpers::BalancesPool,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    account_id: &near_primitives::types::AccountId,
    block: &db_helpers::Block,
    // TODO pagination
    limit: u32,
) -> crate::Result<Vec<ft::schemas::FtBalance>> {
    // todo it's better to query by chunks, this list can bee potentially too big (500+ contracts)
    let query = r"
        SELECT DISTINCT contract_account_id account_id
        FROM coin_events
        WHERE affected_account_id = $1
        ORDER BY contract_account_id
    ";
    let contracts = db_helpers::select_retry_or_panic::<db_helpers::AccountId>(
        pool_balances,
        query,
        &[account_id.to_string()],
    )
    .await?;
    // todo drop this when querying by chunks is implemented for the query above
    tracing::info!(
        target: crate::LOGGER_MSG,
        "get_ft_balances: account {} has {} potential FT balances",
        account_id,
        contracts.len(),
    );

    let mut balances: Vec<ft::schemas::FtBalance> = vec![];
    for contract in contracts {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&contract.account_id) {
            if let Ok(balance) =
                get_ft_balance_by_contract(rpc_client, block, &contract_id, account_id).await
            {
                if balance.amount.0 > 0 {
                    balances.push(balance);
                }
            }
            if balances.len() == limit as usize {
                break;
            }
        }
    }
    Ok(balances)
}

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
    async fn test_ft_balances() {
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let balance = get_ft_balances(&pool_balances, &rpc_client, &account, &block, 10).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balances_empty() {
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("cucumber.near").unwrap();
        let balance = get_ft_balances(&pool_balances, &rpc_client, &account, &block, 10)
            .await
            .unwrap();
        assert!(balance.is_empty());
    }

    #[tokio::test]
    async fn test_ft_balances_skip_zeros() {
        let pool_balances = init_balances_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();
        let balance = get_ft_balances(&pool_balances, &rpc_client, &account, &block, 10).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance_by_contract() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("token.sweat").unwrap();
        let account = near_primitives::types::AccountId::from_str(
            "10241dd91e8d8b6ff7f48ba06eb09c43ee5d5e8f5e7864a477a76161835775c1",
        )
        .unwrap();

        let balance = get_ft_balance_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance_by_contract_zero() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("nexp.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balance_by_contract_no_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_ft_balances_by_contract_other_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_ft_balance_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }
}
