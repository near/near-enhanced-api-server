use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use tracing::instrument;

use super::{data_provider, schemas};
use crate::{db_helpers, modules, types};
#[instrument(skip(pool_explorer, rpc_client, request, block_params))]
#[api_v2_operation(tags(NEAR))]
/// Get user's NEAR balance
///
/// This endpoint returns the NEAR balance of the given `account_id`
/// at the given `block_timestamp_nanos`/`block_height`.
pub async fn get_near_balance(
    pool_explorer: web::Data<db_helpers::ExplorerPool>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::NearBalanceResponse>> {
    let block = db_helpers::checked_get_block(&pool_explorer, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    Ok(Json(
        data_provider::get_near_balance(&pool_explorer, &block, &request.account_id.0).await?,
    ))
}

#[api_v2_operation(tags(NEAR))]
/// Get user's NEAR history
///
/// This endpoint returns the history of NEAR operations
/// for the given `account_id`, `block_timestamp_nanos`/`block_height`.
/// For the next page, use `event_index` of the last item in your previous response.
///
/// **Limitations**
/// We currently provide the history only for the last few months.
/// The history started from genesis will be served soon.
pub async fn get_near_history(
    pool_explorer: web::Data<db_helpers::ExplorerPool>,
    pool_balances: web::Data<db_helpers::BalancesPool>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    let pagination = modules::checked_get_pagination_params(&pagination_params).await?;
    let block = db_helpers::get_block_from_pagination(&pool_explorer, &pagination).await?;
    // we don't need to check whether account exists. If not, we can just return the empty history

    Ok(Json(schemas::HistoryResponse {
        history: data_provider::get_near_history(
            &pool_balances,
            &request.account_id,
            &block,
            &pagination,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}
