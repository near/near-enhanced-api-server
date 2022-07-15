use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use validator::HasLen;

use super::schemas;
use crate::modules::coin::data_provider;
use crate::{db_helpers, errors, modules, types};

#[api_v2_operation]
/// Get user's NEAR balance
///
/// This endpoint returns the NEAR balance of the given account_id
/// for the given timestamp/block_height.
pub async fn get_near_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    request: web::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::NearBalanceResponse>> {
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
    modules::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(
        super::data_provider::get_near_balance(&pool, &block, &request.account_id.0).await?,
    ))
}

#[api_v2_operation]
/// Get user's coin balances
///
/// This endpoint returns all the countable coin balances (including NEAR, FTs, later will add MTs)
/// of the given account_id, for the given timestamp/block_height.
///
/// **Limitations**
/// * For now, we support only the balance for NEAR and FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
/// * We are in the process of supporting Multi Token balances.
/// * We provide only up to 100 items, where recently updated data goes first.
///   Full-featured pagination will be provided later.
pub async fn get_coin_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    // TODO PHASE 2 pagination by index (recently updated go first)
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::CoinBalancesResponse>> {
    types::query_params::check_limit(pagination_params.limit)?;
    let mut pagination = types::query_params::Pagination::from(pagination_params.0);
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
    modules::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let mut balances: Vec<schemas::Coin> = vec![];
    balances.push(
        data_provider::get_near_balance(&pool, &block, &request.account_id.0)
            .await?
            .into(),
    );
    pagination.limit -= 1;

    if pagination.limit > 0 {
        let ft_balances = &mut data_provider::get_coin_balances(
            &pool,
            &rpc_client,
            &block,
            &request.account_id.0,
            &pagination,
        )
        .await?;
        balances.append(ft_balances);
        pagination.limit -= ft_balances.length() as u32;
    }

    Ok(Json(schemas::CoinBalancesResponse {
        balances,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get user's coin balances by contract
///
/// This endpoint returns all the countable coin balances of the given account_id,
/// for the given contract and timestamp/block_height.
/// For FT contract, the response has only 1 item in the list.
/// For MT contracts, there could be several balances (MT support is not ready yet).
///
/// **Limitations**
/// * For now, we support only the balance for FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
/// * We are in the process of supporting Multi Token balances.
pub async fn get_coin_balances_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::BalanceByContractRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::CoinBalancesResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native balance, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
    modules::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let balances = data_provider::get_coin_balances_by_contract(
        &rpc_client,
        &block,
        &request.contract_account_id.0,
        &request.account_id.0,
    )
    .await?;

    Ok(Json(schemas::CoinBalancesResponse {
        balances,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get user's NEAR history
///
/// This endpoint returns the history of operations with NEAR coin
/// for the given account_id, timestamp/block_height.
///
/// **Limitations**
/// * We provide only up to 100 items, where recent updates go first.
///   Full-featured pagination will be provided later.
pub async fn get_near_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    pool_balances: web::Data<db_helpers::DBWrapper>,
    request: web::Path<schemas::BalanceRequest>,
    pagination_params: web::Query<types::query_params::HistoryPaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    let block = db_helpers::get_last_block(&pool).await?;
    modules::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;
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

#[api_v2_operation]
/// Get user's coin history by contract
///
/// This endpoint returns the history of coin operations (FT, other standards)
/// for the given account_id, contract_id, timestamp/block_height.
///
/// **Limitations**
/// * For now, we support only FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
/// * We are in the process of supporting Multi Token history.
/// * We provide only up to 100 items, where recent updates go first.
///   Full-featured pagination will be provided later.
pub async fn get_coin_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::BalanceHistoryRequest>,
    pagination_params: web::Query<types::query_params::HistoryPaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native history, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    let pagination =
        modules::check_and_get_history_pagination_params(&pool, pagination_params.0).await?;
    modules::check_account_exists(&pool, &request.account_id.0, pagination.block_timestamp).await?;

    Ok(Json(schemas::HistoryResponse {
        history: data_provider::get_coin_history(
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

#[api_v2_operation]
/// Get FT contract metadata
///
/// This endpoint returns the metadata for given FT contract and timestamp/block_height.
///
/// **Limitations**
/// * For now, we support only FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
pub async fn get_ft_contract_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::ContractMetadataRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::FtContractMetadataResponse>> {
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(schemas::FtContractMetadataResponse {
        metadata: data_provider::get_ft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}
