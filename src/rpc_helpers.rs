use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryError};

use crate::errors;

const RPC_RETRY_COUNT: usize = 10;

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

pub(crate) fn get_function_call_request(
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

fn clone_request(
    request: &near_jsonrpc_client::methods::query::RpcQueryRequest,
) -> near_jsonrpc_client::methods::query::RpcQueryRequest {
    near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: request.block_reference.clone(),
        request: request.request.clone(),
    }
}

pub(crate) async fn wrapped_call(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    request: near_jsonrpc_client::methods::query::RpcQueryRequest,
    block_height: u64,
    contract_id: &near_primitives::types::AccountId,
) -> crate::Result<near_primitives::views::CallResult> {
    let mut interval = INTERVAL;
    let mut retry_attempt = 0usize;

    tracing::info!(
        target: crate::LOGGER_MSG,
        "RPC request: {:?}\nTo contract:{}, block {}",
        request,
        contract_id,
        block_height
    );

    loop {
        retry_attempt += 1;

        match rpc_client.call(clone_request(&request)).await {
            Ok(response) => {
                return match response.kind {
                    QueryResponseKind::CallResult(result) => Ok(result),
                    // I hope this is unreachable code, so if we meet such case, retry will not help
                    _ => Err(errors::ErrorKind::RPCError(
                        "Unexpected type of the response after CallFunction request".to_string(),
                    )
                    .into()),
                };
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_client::methods::query::RpcQueryError::UnknownAccount { .. },
                ),
            )) => {
                return Err(errors::ErrorKind::InvalidInput(format!(
                    "account_id {} does not exist at block_height {}",
                    contract_id, block_height
                ))
                .into())
            }
            Err(x) => {
                if let Some(RpcQueryError::ContractExecutionError { vm_error, .. }) =
                    x.handler_error()
                {
                    if vm_error.contains("CodeDoesNotExist") || vm_error.contains("MethodNotFound")
                    {
                        // no need to retry this
                        return Err(errors::ErrorKind::InvalidInput(format!(
                            "The account `{}` does not implement any suitable contract at block {}",
                            contract_id, block_height
                        ))
                        .into());
                    }
                }

                tracing::warn!(
                    target: crate::LOGGER_MSG,
                    "Error occurred during {:#?}:\nFailed RPC request: {:?}\nTo contract:{}, block {}\n Retrying in {} milliseconds...",
                    x,
                    request,
                    contract_id,
                    block_height,
                    interval.as_millis(),
                );

                if retry_attempt == RPC_RETRY_COUNT {
                    tracing::warn!(
                        target: crate::LOGGER_MSG,
                        "Failed to perform query to RPC after {} attempts. Stop trying.",
                        RPC_RETRY_COUNT
                    );
                    return Err(x.into());
                }
                tokio::time::sleep(interval).await;
                if interval < MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}
