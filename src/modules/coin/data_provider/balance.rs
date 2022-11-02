use crate::modules::coin;
use crate::{db_helpers, errors, rpc_helpers, types};
use std::str::FromStr;

// todo change to near_balance_events when all the data is collected
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
                    SELECT affected_account_nonstaked_balance + affected_account_staked_balance balance
                    FROM account_changes
                    WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
                    ORDER BY changed_in_block_timestamp DESC
                )
                SELECT * FROM t LIMIT 1
            ",
            &[account_id.to_string(), block.timestamp.to_string()],
        ).await?;

    match balances.first() {
        Some(balance) => Ok(coin::schemas::NearBalanceResponse {
            balance: types::numeric::to_u128(&balance.balance)?.into(),
            metadata: super::metadata::get_near_metadata(),
            block_timestamp_nanos: block.timestamp.into(),
            block_height: block.height.into(),
        }),
        None => Err(errors::ErrorKind::DBError(format!(
            "Could not find the data in account_changes table for account_id {}",
            account_id
        ))
        .into()),
    }
}

// TODO pagination
// todo support legacy contracts
pub(crate) async fn get_coin_balances(
    pool: &sqlx::Pool<sqlx::Postgres>,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &db_helpers::Block,
    account_id: &near_primitives::types::AccountId,
    limit: u32,
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
            limit.to_string(),
        ],
    )
    .await?;
    let mut balances: Vec<coin::schemas::Coin> = vec![];
    for contract in contracts {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&contract.account_id) {
            balances.append(
                &mut get_coin_balances_by_contract(rpc_client, block, &contract_id, account_id)
                    .await?,
            );
        }
    }
    Ok(balances)
}

// TODO PHASE 2 change RPC call to DB call by adding absolute amount values to assets__fungible_token_events
// TODO PHASE 2 add metadata tables to the DB, with periodic autoupdate
pub(crate) async fn get_coin_balances_by_contract(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &db_helpers::Block,
    contract_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> crate::Result<Vec<coin::schemas::Coin>> {
    let (balance, metadata) = (
        get_ft_balance_by_contract(
            rpc_client,
            contract_id.clone(),
            account_id.clone(),
            block.height,
        )
        .await?,
        match super::metadata::get_ft_contract_metadata(
            rpc_client,
            contract_id.clone(),
            block.height,
        )
        .await
        {
            Ok(metadata) => metadata.into(),
            Err(_) => super::metadata::get_default_metadata(),
        },
    );

    Ok(vec![coin::schemas::Coin {
        standard: "nep141".to_string(),
        contract_account_id: Some(contract_id.clone().into()),
        balance: balance.into(),
        metadata,
    }])
}

pub(crate) async fn get_ft_balance_by_contract(
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
            balance: near_coin.balance,
            contract_account_id: None,
            metadata: near_coin.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_near_balance() {
        let pool = init_db().await;
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("tomato.near").unwrap();
        let balance = get_near_balance(&pool, &block, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances() {
        let pool = init_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();
        let balance = get_coin_balances(&pool, &rpc_client, &block, &account, 10).await;
        insta::assert_debug_snapshot!(balance);
    }
    // todo add pagination tests when we finalise how it should look like

    #[tokio::test]
    async fn test_coin_balances_empty() {
        let pool = init_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let balance = get_coin_balances(&pool, &rpc_client, &block, &account, 10)
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

        let balance = get_coin_balances_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances_by_contract_no_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_coin_balances_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }

    #[tokio::test]
    async fn test_coin_balances_by_contract_other_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("patagonita.near").unwrap();

        let balance = get_coin_balances_by_contract(&rpc_client, &block, &contract, &account).await;
        insta::assert_debug_snapshot!(balance);
    }
}
