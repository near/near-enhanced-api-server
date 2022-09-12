use crate::{db_helpers, errors, types};

pub(crate) mod coin;
pub(crate) mod nft;

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

pub(crate) async fn check_and_get_history_pagination_params(
    pool: &sqlx::Pool<sqlx::Postgres>,
    pagination_params: types::query_params::HistoryPaginationParams,
) -> crate::Result<types::query_params::HistoryPagination> {
    types::query_params::check_limit(pagination_params.limit)?;
    let pagination = types::query_params::Pagination::from(pagination_params);
    // if pagination_params.after_block_height.is_some() && pagination_params.after_timestamp_nanos.is_some() {
    //     return Err(errors::ErrorKind::InvalidInput(
    //         "Both block_height and block_timestamp_nanos found. Please provide only one of values"
    //             .to_string(),
    //     )
    //         .into());
    // }
    // TODO PHASE 2 take the block from pagination_params
    let block = db_helpers::get_last_block(pool).await?;
    Ok(types::query_params::HistoryPagination {
        block_height: block.height,
        block_timestamp: block.timestamp,
        limit: pagination.limit,
    })
}

#[cfg(test)]
mod tests {
    use crate::db_helpers;

    pub(crate) async fn init_db() -> sqlx::Pool<sqlx::Postgres> {
        dotenv::dotenv().ok();
        let db_url = &std::env::var("DATABASE_URL").expect("failed to get database url");

        sqlx::PgPool::connect(db_url)
            .await
            .expect("failed to connect to the database")
    }

    pub(crate) fn init_rpc() -> near_jsonrpc_client::JsonRpcClient {
        dotenv::dotenv().ok();
        let rpc_url = &std::env::var("RPC_URL").expect("failed to get RPC url");
        let connector = near_jsonrpc_client::JsonRpcClient::new_client();
        connector.connect(rpc_url)
    }

    pub(crate) fn get_block() -> db_helpers::Block {
        db_helpers::Block {
            timestamp: 1655571176644255779,
            height: 68000000,
        }
    }
}
