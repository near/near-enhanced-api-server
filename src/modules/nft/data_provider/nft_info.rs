use std::str::FromStr;

use crate::modules::nft;
use crate::{db_helpers, errors, rpc_helpers, types};
use serde::{Deserialize, Serialize};

// TODO PHASE 2 pagination by artificial index added to assets__non_fungible_token_events
pub(crate) async fn get_nfts_count(
    db_helpers::ExplorerPool(pool_explorer): &db_helpers::ExplorerPool,
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block: &db_helpers::Block,
    account_id: &near_primitives::types::AccountId,
    limit: u32,
) -> crate::Result<Vec<nft::schemas::NftCount>> {
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

    let info_by_contract = db_helpers::select_retry_or_panic::<super::models::NftCount>(
        pool_explorer,
        query,
        &[
            account_id.to_string(),
            block.timestamp.to_string(),
            limit.to_string(),
        ],
    )
    .await?;

    let mut result: Vec<nft::schemas::NftCount> = vec![];
    for info in info_by_contract {
        if let Ok(contract_id) = near_primitives::types::AccountId::from_str(&info.contract_id) {
            let metadata = super::metadata::get_nft_contract_metadata(
                rpc_client,
                contract_id.clone(),
                block.height,
            )
            .await
            .unwrap_or_else(|_| super::metadata::get_default_nft_contract_metadata());
            result.push(nft::schemas::NftCount {
                contract_account_id: contract_id.into(),
                nft_count: info.count as u32,
                last_updated_at_timestamp_nanos: types::numeric::to_u128(
                    &info.last_updated_at_timestamp,
                )?
                .into(),
                contract_metadata: metadata,
            });
        }
    }
    Ok(result)
}

pub(crate) async fn get_nfts_by_contract(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
    block_height: u64,
    limit: u32,
) -> crate::Result<Vec<nft::schemas::Nft>> {
    // TODO PHASE 2 pagination
    // RPC supports pagination, but the order is defined by the each contract and we can't control it.
    // For now, we are ready to serve only the first page
    // Later, I feel we need to load NFT (each token) metadata to the DB,
    // right after that we can stop using RPC here.
    // Or, maybe we want to delegate this task fully to the contracts?
    let request = rpc_helpers::get_function_call_request(
        block_height,
        contract_id.clone(),
        "nft_tokens_for_owner",
        // https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration
        serde_json::json!({ "account_id": account_id, "from_index": "0", "limit": limit }),
    );
    let response =
        rpc_helpers::wrapped_call(rpc_client, request, block_height, &contract_id).await?;

    let tokens = serde_json::from_slice::<Vec<Token>>(&response.result)?;
    let mut result = vec![];
    for token in tokens {
        result.push(nft::schemas::Nft::try_from(token)?);
    }
    Ok(result)
}

pub(crate) async fn get_nft(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    token_id: String,
    block_height: u64,
) -> crate::Result<nft::schemas::Nft> {
    let request = rpc_helpers::get_function_call_request(
        block_height,
        contract_id.clone(),
        "nft_token",
        serde_json::json!({ "token_id": token_id }),
    );
    let response =
        rpc_helpers::wrapped_call(rpc_client, request, block_height, &contract_id).await?;

    match serde_json::from_slice::<Option<Token>>(&response.result)? {
        None => Err(errors::ErrorKind::InvalidInput(format!(
            "Token `{}` does not exist in contract `{}`, block_height {}",
            token_id, contract_id, block_height
        ))
        .into()),
        Some(token) => nft::schemas::Nft::try_from(token),
    }
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/token.rs
/// Note that token IDs for NFTs are strings on NEAR. It's still fine to use autoincrementing numbers as unique IDs if desired, but they should be stringified. This is to make IDs more future-proof as chain-agnostic conventions and standards arise, and allows for more flexibility with considerations like bridging NFTs across chains, etc.
pub type TokenId = String;

/// In this implementation, the Token struct takes two extensions standards (metadata and approval) as optional fields, as they are frequently used in modern NFTs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: types::AccountId,
    pub metadata: Option<nft::schemas::NftMetadata>,
    pub approved_account_ids: Option<std::collections::HashMap<types::AccountId, u64>>,
}

impl TryFrom<Token> for nft::schemas::Nft {
    type Error = errors::Error;

    fn try_from(token: Token) -> crate::Result<Self> {
        let metadata = token.metadata.ok_or_else(|| {
            errors::ErrorKind::ContractError(
                "The contract did not provide NFT Metadata which is a required part of NFT NEP 171"
                    .to_string(),
            )
        })?;

        Ok(Self {
            token_id: token.token_id,
            owner_account_id: token.owner_id.0.to_string(),
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;

    #[tokio::test]
    async fn test_nfts_count() {
        let pool_explorer = init_explorer_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("blondjesus.near").unwrap();

        let nft_count = get_nfts_count(&pool_explorer, &rpc_client, &block, &account, 10).await;
        insta::assert_debug_snapshot!(nft_count);
    }

    #[tokio::test]
    async fn test_nfts_count_empty() {
        let pool_explorer = init_explorer_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("cucumber.near").unwrap();

        let nft_count = get_nfts_count(&pool_explorer, &rpc_client, &block, &account, 10)
            .await
            .unwrap();
        assert!(nft_count.is_empty());
    }

    #[tokio::test]
    async fn test_nfts_count_with_contracts_with_no_metadata() {
        let pool_explorer = init_explorer_db().await;
        let rpc_client = init_rpc();
        let block = db_helpers::Block {
            timestamp: 1655571176644255779,
            height: 68000000,
        };
        let account = near_primitives::types::AccountId::from_str("vlad.near").unwrap();

        let nft_count = get_nfts_count(&pool_explorer, &rpc_client, &block, &account, 10).await;
        insta::assert_debug_snapshot!(nft_count);
    }

    #[tokio::test]
    async fn test_nfts_count_with_no_failed_receipts_in_result() {
        let pool_explorer = init_explorer_db().await;
        let rpc_client = init_rpc();
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("kbneoburner3.near").unwrap();

        let nft_count = get_nfts_count(&pool_explorer, &rpc_client, &block, &account, 10).await;
        insta::assert_debug_snapshot!(nft_count);
    }

    #[tokio::test]
    async fn test_nfts_by_contract() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract =
            near_primitives::types::AccountId::from_str("billionairebullsclub.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("olenavorobei.near").unwrap();

        let nfts = get_nfts_by_contract(&rpc_client, contract, account, block.height, 4).await;
        insta::assert_debug_snapshot!(nfts);
    }

    #[tokio::test]
    async fn test_nft() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "415815:1".to_string();

        let nft = get_nft(&rpc_client, contract, token, block.height).await;
        insta::assert_debug_snapshot!(nft);
    }

    #[tokio::test]
    async fn test_nft_does_not_exist() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "no_such_token".to_string();

        let nft = get_nft(&rpc_client, contract, token, block.height).await;
        insta::assert_debug_snapshot!(nft);
    }
}
