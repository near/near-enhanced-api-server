use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use super::{data_provider, schemas};
use crate::{db_helpers, modules, types};

#[api_v2_operation(tags(NEAR))]
/// Get user's NEAR balance
///
/// This endpoint returns the NEAR balance of the given `account_id`
/// at the given `timestamp`/`block_height`.
pub async fn get_near_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::NearBalanceResponse>> {
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    Ok(Json(
        data_provider::get_near_balance(&pool, &block, &request.account_id.0).await?,
    ))
}

#[api_v2_operation(tags(NEAR))]
/// Get user's NEAR history
///
/// This endpoint returns the history of operations with NEAR coins
/// for the given `account_id`, `timestamp`/`block_height`.
///
/// **Limitations**
/// * We currently provide the most recent 100 items.
///   Full-featured pagination will be provided in an upcoming update.
pub async fn get_near_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    pool_balances: web::Data<db_helpers::DBWrapper>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    pagination_params: web::Query<types::query_params::HistoryPaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    let block = db_helpers::get_last_block(&pool).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;
    let pagination =
        modules::check_and_get_history_pagination_params(&pool, pagination_params.0).await?;

    Ok(Json(schemas::HistoryResponse {
        history: data_provider::get_near_history(
            &pool_balances.pool,
            &request.account_id,
            &pagination,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}
