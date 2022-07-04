use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryError};

use crate::{api_models, errors, types, utils};

pub(crate) async fn get_ft_balance(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
    block_height: u64,
) -> api_models::Result<u128> {
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id: contract_id.clone(),
            method_name: "ft_balance_of".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({
                    "account_id": account_id,
                })
                .to_string()
                .into_bytes(),
            ),
        },
    };

    // todo put this code into function, now I duplicate it everywhere
    let response = match rpc_client.call(request).await {
        Ok(x) => x,
        Err(x) => {
            if let Some(RpcQueryError::ContractExecutionError { vm_error, .. }) = x.handler_error()
            {
                if vm_error.contains("CodeDoesNotExist") || vm_error.contains("MethodNotFound") {
                    return Err(errors::contract_not_found(&contract_id, block_height).into());
                }
            }
            return Err(x.into());
        }
    };

    match response.kind {
        QueryResponseKind::CallResult(result) => {
            Ok(serde_json::from_slice::<types::U128>(&result.result)?.0)
        }
        _ => Err(errors::ErrorKind::RPCError(
            "Unexpected type of the response after CallFunction request".to_string(),
        )
        .into()),
    }
}

pub(crate) async fn get_ft_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    block_height: u64,
) -> api_models::Result<api_models::FtContractMetadata> {
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id: contract_id.clone(),
            method_name: "ft_metadata".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({}).to_string().into_bytes(),
            ),
        },
    };

    let response = match rpc_client.call(request).await {
        Ok(x) => x,
        Err(x) => {
            if let Some(RpcQueryError::ContractExecutionError { vm_error, .. }) = x.handler_error()
            {
                if vm_error.contains("CodeDoesNotExist") || vm_error.contains("MethodNotFound") {
                    return Err(errors::contract_not_found(&contract_id, block_height).into());
                }
            }
            return Err(x.into());
        }
    };

    match response.kind {
        QueryResponseKind::CallResult(result) => {
            let metadata = serde_json::from_slice::<types::FungibleTokenMetadata>(&result.result)?;
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
        _ => Err(errors::ErrorKind::RPCError(
            "Unexpected type of the response after CallFunction request".to_string(),
        )
        .into()),
    }
}

pub(crate) async fn get_nft_general_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    block_height: u64,
) -> api_models::Result<api_models::NftContractMetadata> {
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id: contract_id.clone(),
            method_name: "nft_metadata".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({}).to_string().into_bytes(),
            ),
        },
    };

    let response = match rpc_client.call(request).await {
        Ok(x) => x,
        Err(x) => {
            if let Some(RpcQueryError::ContractExecutionError { vm_error, .. }) = x.handler_error()
            {
                if vm_error.contains("CodeDoesNotExist") || vm_error.contains("MethodNotFound") {
                    return Err(errors::contract_not_found(&contract_id, block_height).into());
                }
            }
            return Err(x.into());
        }
    };

    match response.kind {
        QueryResponseKind::CallResult(result) => {
            api_models::NftContractMetadata::try_from(serde_json::from_slice::<
                types::NFTContractMetadata,
            >(&result.result)?)
        }
        _ => Err(errors::ErrorKind::RPCError(
            "Unexpected type of the response after CallFunction request".to_string(),
        )
        .into()),
    }
}

pub(crate) async fn get_nfts(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
    block_height: u64,
    limit: u32,
) -> api_models::Result<Vec<api_models::NonFungibleToken>> {
    // todo pagination (can wait for phase 2)
    // RPC supports pagination, but the order is defined by the each contract and we can't control it.
    // For now, we are ready to serve only the first page
    // Later, I feel we need to load NFT (each token) metadata to the DB,
    // right after that we can stop using RPC here.
    // Or, maybe we want to delegate this task fully to the contracts?
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id: contract_id.clone(),
            method_name: "nft_tokens_for_owner".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                // https://nomicon.io/Standards/Tokens/NonFungibleToken/Enumeration
                serde_json::json!({ "account_id": account_id, "from_index": "0", "limit": limit })
                    .to_string()
                    .into_bytes(),
            ),
        },
    };

    let response = match rpc_client.call(request).await {
        Ok(x) => x,
        Err(x) => {
            if let Some(RpcQueryError::ContractExecutionError { vm_error, .. }) = x.handler_error()
            {
                if vm_error.contains("CodeDoesNotExist") || vm_error.contains("MethodNotFound") {
                    return Err(errors::contract_not_found(&contract_id, block_height).into());
                }
            }
            return Err(x.into());
        }
    };

    match response.kind {
        QueryResponseKind::CallResult(result) => {
            let tokens = serde_json::from_slice::<Vec<types::Token>>(&result.result)?;
            let mut result = vec![];
            for token in tokens {
                result.push(api_models::NonFungibleToken::try_from(token)?);
            }
            Ok(result)
        }
        _ => Err(errors::ErrorKind::RPCError(
            "Unexpected type of the response after CallFunction request".to_string(),
        )
        .into()),
    }
}

pub(crate) async fn get_nft_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    token_id: String,
    block_height: u64,
) -> api_models::Result<api_models::NonFungibleToken> {
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id: contract_id.clone(),
            method_name: "nft_token".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({ "token_id": token_id })
                    .to_string()
                    .into_bytes(),
            ),
        },
    };

    let response = match rpc_client.call(request).await {
        Ok(x) => x,
        Err(x) => {
            if let Some(RpcQueryError::ContractExecutionError { vm_error, .. }) = x.handler_error()
            {
                if vm_error.contains("CodeDoesNotExist") || vm_error.contains("MethodNotFound") {
                    return Err(errors::contract_not_found(&contract_id, block_height).into());
                }
            }
            return Err(x.into());
        }
    };

    match response.kind {
        QueryResponseKind::CallResult(result) => api_models::NonFungibleToken::try_from(
            serde_json::from_slice::<types::Token>(&result.result)?,
        ),
        _ => Err(errors::ErrorKind::RPCError(
            "Unexpected type of the response after CallFunction request".to_string(),
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn init() -> (near_jsonrpc_client::JsonRpcClient, u64) {
        (
            near_jsonrpc_client::JsonRpcClient::connect("https://archival-rpc.mainnet.near.org"),
            68000000,
        )
    }

    #[actix_rt::test]
    async fn test_ft_balance() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();
        let account = near_primitives::types::AccountId::from_str("cgarls.near").unwrap();

        let balance = get_ft_balance(&rpc_client, contract, account, block_height)
            .await
            .unwrap();
        assert_eq!(17201878399999996928, balance);
    }

    #[actix_rt::test]
    async fn test_ft_metadata() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();

        let metadata = get_ft_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[actix_rt::test]
    async fn test_ft_metadata_no_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();

        let metadata = get_ft_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[actix_rt::test]
    async fn test_ft_metadata_other_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();

        let metadata = get_ft_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[actix_rt::test]
    async fn test_nft_general_metadata() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();

        let metadata = get_nft_general_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[actix_rt::test]
    async fn test_nft_general_metadata_no_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();

        let metadata = get_nft_general_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[actix_rt::test]
    async fn test_nft_metadata_other_contract_deployed() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();

        let metadata = get_nft_general_metadata(&rpc_client, contract, block_height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[actix_rt::test]
    async fn test_nft_list() {
        let (rpc_client, block_height) = init();
        let contract =
            near_primitives::types::AccountId::from_str("billionairebullsclub.near").unwrap();
        let account = near_primitives::types::AccountId::from_str("olenavorobei.near").unwrap();

        let nfts = get_nfts(&rpc_client, contract, account, block_height, 4).await;
        insta::assert_debug_snapshot!(nfts);
    }

    #[actix_rt::test]
    async fn test_nft_metadata() {
        let (rpc_client, block_height) = init();
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "415815:1".to_string();

        let nfts = get_nft_metadata(&rpc_client, contract, token, block_height).await;
        insta::assert_debug_snapshot!(nfts);
    }
}
