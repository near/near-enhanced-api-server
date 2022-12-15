use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use super::{data_provider, schemas};
use crate::{db_helpers, errors, modules, types};

#[api_v2_operation(tags(FT))]
/// Get user's FT balances
///
/// This endpoint returns all non-zero FT balances of the given `account_id`,
/// at the given `block_timestamp_nanos`/`block_height`.
///
/// **Limitations**
/// * We may accidentally skip the contract if the balances wasn't updated in the last few months,
///   we are still working on the full coverage.
/// * We currently provide up to 100 items.
///   Full-featured pagination will be provided soon.
pub async fn get_ft_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    pool_balances: web::Data<db_helpers::DBWrapper>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    // TODO pagination
    limit_params: web::Query<types::query_params::LimitParams>,
) -> crate::Result<Json<schemas::FtBalancesResponse>> {
    let limit = types::query_params::checked_get_limit(limit_params.limit)?;
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    let balances = data_provider::get_ft_balances(
        &pool_balances.pool,
        &rpc_client,
        &request.account_id.0,
        &block,
        limit,
    )
    .await?;

    Ok(Json(schemas::FtBalancesResponse {
        balances,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation(tags(FT))]
/// Get user's FT balance by contract
///
/// This endpoint returns FT balance of the given `account_id`,
/// for the given `contract_account_id` and `block_timestamp_nanos`/`block_height`.
pub async fn get_ft_balance_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceByContractRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::FtBalanceByContractResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native balance, please use `/accounts/{account_id}/balances/NEAR`".to_string(),
        )
        .into());
    }
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    let balance = data_provider::get_ft_balance_by_contract(
        &rpc_client,
        &block,
        &request.contract_account_id.0,
        &request.account_id.0,
    )
    .await?;

    Ok(Json(schemas::FtBalanceByContractResponse {
        balance,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation(tags(FT))]
/// Get user's FT history by contract
///
/// This endpoint returns the history of FT operations
/// for the given `account_id`, `contract_account_id`, `block_timestamp_nanos`/`block_height`.
///
/// **Limitations**
/// We currently provide the history only for the last few months.
/// The history started from genesis will be served soon.
pub async fn get_ft_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    pool_balances: web::Data<db_helpers::DBWrapper>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::HistoryRequest>,
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native history, please use `/accounts/{account_id}/balances/NEAR/history`"
                .to_string(),
        )
        .into());
    }
    let pagination = modules::checked_get_pagination_params(&pagination_params).await?;
    let block = db_helpers::checked_get_block_from_pagination(&pool, &pagination).await?;
    // we don't need to check whether account exists. If not, we can just return the empty history

    Ok(Json(schemas::HistoryResponse {
        history: data_provider::get_ft_history(
            &pool,
            &pool_balances.pool,
            &rpc_client,
            &request.contract_account_id.0,
            &request.account_id.0,
            &block,
            &pagination,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation(tags(FT))]
/// Get FT metadata
///
/// This endpoint returns the metadata for the given `contract_account_id`, `block_timestamp_nanos`/`block_height`.
pub async fn get_ft_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::ContractMetadataRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::FtContractMetadataResponse>> {
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;

    Ok(Json(schemas::FtContractMetadataResponse {
        metadata: data_provider::get_ft_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}
