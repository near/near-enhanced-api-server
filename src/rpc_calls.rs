use near_jsonrpc_primitives::types::query::QueryResponseKind;

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
            account_id: contract_id,
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

    match rpc_client.call(request).await?.kind {
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
            account_id: contract_id,
            method_name: "ft_metadata".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({}).to_string().into_bytes(),
            ),
        },
    };

    match rpc_client.call(request).await?.kind {
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
            account_id: contract_id,
            method_name: "nft_metadata".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({}).to_string().into_bytes(),
            ),
        },
    };

    match rpc_client.call(request).await?.kind {
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

pub(crate) async fn get_nft_count(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    account_id: near_primitives::types::AccountId,
    block_height: u64,
) -> api_models::Result<u32> {
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id: contract_id,
            method_name: "nft_supply_for_owner".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({ "account_id": account_id })
                    .to_string()
                    .into_bytes(),
            ),
        },
    };

    match rpc_client.call(request).await?.kind {
        QueryResponseKind::CallResult(result) => {
            let a = serde_json::from_slice::<String>(&result.result)?;
            let x: u32 = a.parse().map_err(|e| {
                errors::ErrorKind::InternalError(format!("Failed to parse u32 {}", e))
                //.into()
            })?;
            Ok(x)
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
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockReference::BlockId(
            near_primitives::types::BlockId::Height(block_height),
        ),
        request: near_primitives::views::QueryRequest::CallFunction {
            account_id: contract_id,
            method_name: "nft_tokens_for_owner".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({ "account_id": account_id, "from_index": "0", "limit": limit })
                    .to_string()
                    .into_bytes(),
            ),
        },
    };

    match rpc_client.call(request).await?.kind {
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
            account_id: contract_id,
            method_name: "nft_token".to_string(),
            args: near_primitives::types::FunctionArgs::from(
                serde_json::json!({ "token_id": token_id })
                    .to_string()
                    .into_bytes(),
            ),
        },
    };

    match rpc_client.call(request).await?.kind {
        QueryResponseKind::CallResult(result) => api_models::NonFungibleToken::try_from(
            serde_json::from_slice::<types::Token>(&result.result)?,
        ),
        _ => Err(errors::ErrorKind::RPCError(
            "Unexpected type of the response after CallFunction request".to_string(),
        )
        .into()),
    }
}
