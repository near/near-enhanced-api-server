use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use super::{data_provider, schemas};
use crate::{db_helpers, errors, modules, types};

#[api_v2_operation(tags(FT))]
/// Get user's FT balances
///
/// This endpoint returns FT balances
/// of the given `account_id`, at the given `timestamp`/`block_height`.
///
/// **Limitations**
/// * For now, we only support the balance for FT contracts that implement the Events NEP standard.
///   We are working on a solution to support other FT contracts, including `wrap.near` and bridged tokens.
/// * We currently provide the most recent 100 items.
///   Full-featured pagination will be provided in an upcoming update.
pub async fn get_ft_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    // TODO PHASE 2 pagination by index (recently updated go first)
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::FtBalancesResponse>> {
    types::query_params::check_limit(pagination_params.limit)?;
    let pagination = types::query_params::Pagination::from(pagination_params.0);
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    let balances = data_provider::get_ft_balances(
        &pool,
        &rpc_client,
        &block,
        &request.account_id.0,
        &pagination,
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
/// for the given `contract_account_id` and `timestamp`/`block_height`.
///
/// **Limitations**
/// * For now, we support only the balance for FT contracts that implement the Events NEP standard.
///   We are working on a solution to support other FT contracts, including `wrap.near` and bridged tokens.
pub async fn get_ft_balance_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceByContractRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::FtBalanceByContractResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native balance, please use the other endpoint".to_string(),
        )
        .into());
    }
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
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
/// for the given `account_id`, `contract_id`, `timestamp`/`block_height`.
///
/// **Limitations**
/// * For now, we support only FT contracts that implement the Events NEP standard.
///   We are working on a solution to support other FT contracts, including `wrap.near` and bridged tokens.
/// * We currently provide the most recent 100 items.
///   Full-featured pagination will be provided in an upcoming update.
pub async fn get_ft_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::HistoryRequest>,
    pagination_params: web::Query<types::query_params::HistoryPaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native history, please use the other endpoint".to_string(),
        )
        .into());
    }
    let pagination =
        modules::check_and_get_history_pagination_params(&pool, pagination_params.0).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, pagination.block_height)
        .await?;

    Ok(Json(schemas::HistoryResponse {
        history: data_provider::get_ft_history(
            &pool,
            &rpc_client,
            &request.contract_account_id.0,
            &request.account_id.0,
            &pagination,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(pagination.block_timestamp),
        block_height: types::U64::from(pagination.block_height),
    }))
}

#[api_v2_operation(tags(FT))]
/// Get FT metadata
///
/// This endpoint returns the metadata for a given FT contract and `timestamp`/`block_height`.
///
/// **Limitations**
/// * For now, we support only FT contracts that implement the Events NEP standard.
///   We are working on a solution to support other FT contracts, including `wrap.near` and bridged tokens.
pub async fn get_ft_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::ContractMetadataRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::FtContractMetadataResponse>> {
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;

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
