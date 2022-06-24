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
) -> api_models::Result<types::NFTContractMetadata> {
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
        QueryResponseKind::CallResult(result) => Ok(serde_json::from_slice::<
            types::NFTContractMetadata,
        >(&result.result)?),
        _ => Err(errors::ErrorKind::RPCError(
            "Unexpected type of the response after CallFunction request".to_string(),
        )
        .into()),
    }
}
