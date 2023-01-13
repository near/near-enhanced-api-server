use tracing::instrument;

use crate::{errors, types};

pub(crate) mod ft;
pub(crate) mod native;
pub(crate) mod nft;

#[instrument(skip_all)]
pub(crate) async fn check_account_exists(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    account_id: &near_primitives::types::AccountId,
    block_height: u64,
) -> crate::Result<()> {
    let request = near_jsonrpc_client::methods::query::RpcQueryRequest {
        block_reference: near_primitives::types::BlockId::Height(block_height).into(),
        request: near_primitives::views::QueryRequest::ViewAccount {
            account_id: account_id.clone(),
        },
    };
    for _ in 0..5 {
        match rpc_client.call(&request).await {
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                    near_jsonrpc_client::methods::query::RpcQueryError::UnknownAccount { .. },
                ),
            )) => {
                return Err(errors::ErrorKind::InvalidInput(format!(
                    "account_id {} does not exist at block_height {}",
                    account_id, block_height
                ))
                .into())
            }
            Err(err) => {
                tracing::warn!(target: crate::LOGGER_MSG, "Checking account existence via JSON RPC failed with: {:?}. Re-trying immediatelly", err);
                continue;
            }
            Ok(response) => {
                if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(_) =
                    response.kind
                {
                    return Ok(());
                } else {
                    tracing::warn!(target: crate::LOGGER_MSG, "Checking account existence returned invalid response: {:?}. Re-trying immediatelly", response);
                    continue;
                }
            }
        }
    }
    Err(errors::ErrorKind::InternalError(format!(
        "could not check if account_id {} exists after several attemps",
        account_id
    ))
    .into())
}

/// Validates pagination_params received from the user
pub(crate) async fn checked_get_pagination_params(
    pagination_params: &types::query_params::PaginationParams,
) -> crate::Result<types::query_params::Pagination> {
    Ok(types::query_params::Pagination {
        limit: types::query_params::checked_get_limit(pagination_params.limit)?,
        after_event_index: match pagination_params.after_event_index {
            None => None,
            Some(index) => {
                if index.0 < crate::MIN_EVENT_INDEX {
                    return Err(errors::ErrorKind::InvalidInput(format!(
                        "after_event_index {} is too low. Please copy event_index value from the last item in your previous response",
                        index.0
                    ))
                        .into());
                }
                Some(index.0)
            }
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::db_helpers;

    pub(crate) async fn init_explorer_db() -> db_helpers::ExplorerPool {
        dotenv::dotenv().ok();
        let db_url = &std::env::var("EXPLORER_DATABASE_URL").expect("failed to get database url");

        db_helpers::ExplorerPool(
            sqlx::PgPool::connect(db_url).await.expect(
                "failed to connect to the database from EXPLORER_DATABASE_URL env variable",
            ),
        )
    }

    pub(crate) async fn init_balances_db() -> db_helpers::BalancesPool {
        dotenv::dotenv().ok();
        let db_url_balances =
            &std::env::var("BALANCES_DATABASE_URL").expect("failed to get database url");
        db_helpers::BalancesPool(sqlx::PgPool::connect(db_url_balances).await.expect(
            "failed to connect to the balances database from BALANCES_DATABASE_URL env variable",
        ))
    }

    pub(crate) fn init_rpc() -> near_jsonrpc_client::JsonRpcClient {
        dotenv::dotenv().ok();
        let rpc_url = &std::env::var("RPC_URL").expect("failed to get RPC url");
        let connector = near_jsonrpc_client::JsonRpcClient::new_client();
        connector.connect(rpc_url)
    }

    pub(crate) fn get_block() -> db_helpers::Block {
        db_helpers::Block {
            timestamp: 1670867692546051383, // December 12, 2022
            height: 80500000,
        }
    }
}
