use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryError};

use crate::{api_models, errors, types, utils};

pub(crate) async fn get_ft_balance(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
    block_height: u64,
) -> api_models::Result<u128> {
    let request = get_function_call_request(
        block_height,
        contract_id.clone(),
        "ft_balance_of",
        serde_json::json!({ "account_id": account_id }),
    );
    let response = wrapped_call(rpc_client, request, block_height, &contract_id).await?;
    Ok(serde_json::from_slice::<types::U128>(&response.result)?.0)
}

pub(crate) async fn get_ft_contract_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    block_height: u64,
) -> api_models::Result<api_models::FtContractMetadata> {
    let request = get_function_call_request(
        block_height,
        contract_id.clone(),
        "ft_metadata",
        serde_json::json!({}),
    );
    let response = wrapped_call(rpc_client, request, block_height, &contract_id).await?;

    let metadata = serde_json::from_slice::<types::FungibleTokenMetadata>(&response.result)?;
    Ok(api_models::FtContractMetadata {
        spec: metadata.spec,
        name: metadata.name,
        symbol: metadata.symbol,
        icon: metadata.icon,
        decimals: metadata.decimals,
        reference: metadata.reference,
        reference_hash: utils::base64_to_string(&metadata.reference_hash)?,
    })
}

pub(crate) async fn get_nft_contract_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    block_height: u64,
) -> api_models::Result<api_models::NftContractMetadata> {
    let request = get_function_call_request(
        block_height,
        contract_id.clone(),
        "nft_metadata",
        serde_json::json!({}),
    );
    let response = match wrapped_call(rpc_client, request, block_height, &contract_id).await {
        Ok(response) => response,
        Err(err) => {
            println!("{}", err.message);
            if err
                .message
                .contains("called `Option::unwrap()` on a `None` value")
            {
                return Err(errors::ErrorKind::ContractError(
                    "The contract did not provide NFT Metadata which is a required part of NFT NEP 171".to_string(),
                )
                    .into());
            }
            return Err(err);
        }
    };

    api_models::NftContractMetadata::try_from(serde_json::from_slice::<types::NFTContractMetadata>(
        &response.result,
    )?)
}

pub(crate) async fn get_nft_collection(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
    block_height: u64,
    limit: u32,
) -> api_models::Result<Vec<api_models::NonFungibleToken>> {
    // TODO PHASE 2 pagination
    // RPC supports pagination, but the order is defined by the each contract and we can't control it.
    // For now, we are ready to serve only the first page
    // Later, I feel we need to load NFT (each token) metadata to the DB,
    // right after that we can stop using RPC here.
    // Or, maybe we want to delegate this task fully to the contracts?
    let request = get_function_call_request(
        block_height,
        contract_id.clone(),
        "nft_tokens_for_owner",
        // https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration
        serde_json::json!({ "account_id": account_id, "from_index": "0", "limit": limit }),
    );
    let response = wrapped_call(rpc_client, request, block_height, &contract_id).await?;

    let tokens = serde_json::from_slice::<Vec<types::Token>>(&response.result)?;
    let mut result = vec![];
    for token in tokens {
        result.push(api_models::NonFungibleToken::try_from(token)?);
    }
    Ok(result)
}

pub(crate) async fn get_nft_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    token_id: String,
    block_height: u64,
) -> api_models::Result<api_models::NonFungibleToken> {
    let request = get_function_call_request(
        block_height,
        contract_id.clone(),
        "nft_token",
        serde_json::json!({ "token_id": token_id }),
    );
    let response = wrapped_call(rpc_client, request, block_height, &contract_id).await?;

    match serde_json::from_slice::<Option<types::Token>>(&response.result)? {
        None => Err(errors::ErrorKind::InvalidInput(format!(
            "Token `{}` does not exist in contract `{}`, block_height {}",
            token_id, contract_id, block_height
        ))
        .into()),
        Some(token) => api_models::NonFungibleToken::try_from(token),
    }
}

fn get_function_call_request(
    block_height: u64,
    account_id: near_primitives::types::AccountId,
    method_name: &str,
    args: serde_json::Value,
) -> near_jsonrpc_client::methods::query::RpcQueryRequest {
    near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id,
            method_name: method_name.to_string(),
            args: near_primitives::types::FunctionArgs::from(args.to_string().into_bytes()),
        },
    }
}

async fn wrapped_call(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    request: near_jsonrpc_client::methods::query::RpcQueryRequest,
    block_height: u64,
    contract_id: &near_primitives::types::AccountId,
) -> api_models::Result<near_primitives::views::CallResult> {
    tracing::info!(
        target: utils::LOGGER_MSG,
        "RPC request: {:?}\nTo contract:{}, block {}",
        request,
        contract_id,
        block_height
    );
    match rpc_client.call(request).await {
        Ok(response) => match response.kind {
            QueryResponseKind::CallResult(result) => Ok(result),
            _ => Err(errors::ErrorKind::RPCError(
                "Unexpected type of the response after CallFunction request".to_string(),
            )
            .into()),
        },
        Err(x) => {
            if let Some(RpcQueryError::ContractExecutionError { vm_error, .. }) = x.handler_error()
            {
                if vm_error.contains("CodeDoesNotExist") || vm_error.contains("MethodNotFound") {
                    return Err(errors::contract_not_found(contract_id, block_height).into());
                }
            }
            Err(x.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn init() -> (near_jsonrpc_client::JsonRpcClient, u64) {
        dotenv::dotenv().ok();
        let rpc_url = &std::env::var("RPC_URL").expect("failed to get RPC url");
        let connector = near_jsonrpc_client::JsonRpcClient::new_client();
        (connector.connect(rpc_url), 68000000)
    }

    #[tokio::test]
    async fn test_ft_balance() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();
        let account = near_primitives::types::AccountId::from_str("cgarls.near").unwrap();

        let balance = get_ft_balance(&rpc_client, contract, account, block_height)
            .await
            .unwrap();
        assert_eq!(17201878399999996928, balance);
    }

    #[tokio::test]
    async fn test_ft_contract_metadata() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();

        let metadata = get_ft_contract_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_ft_contract_metadata_no_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();

        let metadata = get_ft_contract_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_ft_contract_metadata_other_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();

        let metadata = get_ft_contract_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_contract_metadata() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_contract_metadata_no_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_contract_metadata_other_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_contract_metadata_broken_contract() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("nft.nearapps.near").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_collection() {
        let (rpc_client, block_height) = init();
        let contract =
            near_primitives::types::AccountId::from_str("billionairebullsclub.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("olenavorobei.near").unwrap();

        let nfts = get_nft_collection(&rpc_client, contract, account, block_height, 4).await;
        insta::assert_debug_snapshot!(nfts);
    }

    #[tokio::test]
    async fn test_nft_metadata() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "415815:1".to_string();

        let nft = get_nft_metadata(&rpc_client, contract, token, block_height).await;
        insta::assert_debug_snapshot!(nft);
    }

    #[tokio::test]
    async fn test_nft_metadata_token_does_not_exist() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "no_such_token".to_string();

        let nft = get_nft_metadata(&rpc_client, contract, token, block_height).await;
        insta::assert_debug_snapshot!(nft);
    }
}
