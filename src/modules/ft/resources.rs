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
/// This endpoint scans all the FT contracts.
/// We currently provide up to 100 results, which covers almost all the potential situations.
/// Anyway, full-featured pagination will be provided soon.
pub async fn get_ft_balances(
    pool_explorer: web::Data<db_helpers::ExplorerPool>,
    pool_balances: web::Data<db_helpers::BalancesPool>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    // TODO pagination
    limit_params: web::Query<types::query_params::LimitParams>,
) -> crate::Result<Json<schemas::FtBalancesResponse>> {
    let limit = types::query_params::checked_get_limit(limit_params.limit)?;
    let block = db_helpers::checked_get_block(&pool_explorer, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    let balances = data_provider::get_ft_balances(
        &pool_balances,
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
    pool_explorer: web::Data<db_helpers::ExplorerPool>,
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
    let block = db_helpers::checked_get_block(&pool_explorer, &block_params).await?;
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
/// for the given `account_id`, `contract_account_id`.
/// For the next page, use `event_index` of the last item in your previous response.
pub async fn get_ft_history(
    pool_explorer: web::Data<db_helpers::ExplorerPool>,
    pool_balances: web::Data<db_helpers::BalancesPool>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::HistoryRequest>,
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::FtHistoryResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native history, please use `/accounts/{account_id}/balances/NEAR/history`"
                .to_string(),
        )
        .into());
    }
    let pagination = modules::checked_get_pagination_params(&pagination_params).await?;
    let block = db_helpers::get_block_from_pagination(&pool_explorer, &pagination).await?;
    // we don't need to check whether account exists. If not, we can just return the empty history

    Ok(Json(schemas::FtHistoryResponse {
        history: data_provider::get_ft_history(
            &pool_explorer,
            &pool_balances,
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
    pool_explorer: web::Data<db_helpers::ExplorerPool>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::ContractMetadataRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::FtContractMetadataResponse>> {
    let block = db_helpers::checked_get_block(&pool_explorer, &block_params).await?;

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
