use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use super::{data_provider, schemas};
use crate::{db_helpers, errors, modules, types};

#[api_v2_operation(tags(Coins))]
/// Get user's NEAR balance
///
/// This endpoint returns the NEAR balance of the given `account_id`
/// at the given `block_timestamp_nanos`/`block_height`.
pub async fn get_near_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::NearBalanceResponse>> {
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    Ok(Json(
        data_provider::get_near_balance(&pool, &block, &request.account_id.0).await?,
    ))
}

#[api_v2_operation(tags(Coins))]
/// Get user's coin balances
///
/// This endpoint returns all the countable coin balances (including NEAR, FTs)
/// of the given `account_id`, at the given `block_timestamp_nanos`/`block_height`.
/// todo it's not great, we can't keep sorting + pagination when we add MT here
/// NEAR balance always goes first, then the balances are sorted alphabetically by `contract_account_id`.
///
/// You can also provide `limit` (max 100) and `after_contract_account_id`.
/// `after_contract_account_id` is the way to paginate through the values.
///
/// Some pagination notes: leave `after_contract_account_id` empty at your first request;
/// then, take `contract_account_id` of the last provided item in your previous response.
///
/// **Limitations**
/// * We are in the process of supporting Multi Token balances.
pub async fn get_coin_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    limit_params: web::Query<types::query_params::LimitParams>,
) -> crate::Result<Json<schemas::CoinBalancesResponse>> {
    let mut limit = types::query_params::checked_get_limit(limit_params.limit)?;
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    let mut balances: Vec<schemas::Coin> = vec![];
    balances.push(
        data_provider::get_near_balance(&pool, &block, &request.account_id.0)
            .await?
            .into(),
    );
    limit -= 1;

    if limit > 0 {
        let ft_balances = &mut data_provider::get_coin_balances(
            &pool,
            &rpc_client,
            &block,
            &request.account_id.0,
            limit,
        )
        .await?;
        balances.append(ft_balances);
    }

    Ok(Json(schemas::CoinBalancesResponse {
        balances,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation(tags(Coins))]
/// Get user's coin balances by contract
///
/// This endpoint returns all the countable coin balances of the given `account_id`,
/// for the given `contract_account_id` and `block_timestamp_nanos`/`block_height`.
/// For FT contracts, the response has only 1 item in the list.
/// For MT contracts, there could be several balances (MT support is still under development).
/// todo change interface for ft/mt
///
/// **Limitations**
/// * We are in the process of supporting Multi Token balances.
pub async fn get_coin_balances_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceByContractRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::CoinBalancesResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native balance, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

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

#[api_v2_operation(tags(Coins))]
/// Get user's NEAR history
///
/// This endpoint returns the history of native NEAR operations,
/// for the given `account_id`, going from the latest to the earliest events.
/// For the next page, use `after_event_index` and pass there the index from the last event from the previous page.
///
/// **Limitations**
/// We are in the process of collecting the data. We support only last 3 months history for now.
pub async fn get_near_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    pool_balances: web::Data<db_helpers::DBWrapper>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::BalanceRequest>,
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    let pagination = modules::checked_get_pagination_params(&pagination_params).await?;
    let block = db_helpers::checked_get_block_from_pagination(&pool, &pagination).await?;
    // we don't need to check whether account exists. If not, we can just return the empty history

    Ok(Json(schemas::HistoryResponse {
        history: data_provider::get_near_history(
            &pool_balances.pool,
            &request.account_id.0,
            &block,
            &pagination,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation(tags(Coins))]
/// Get user's coin history by contract
///
/// This endpoint returns the history of coin operations (FT, other standards)
/// for the given `account_id`, `contract_id`, going from the latest to the earliest events.
/// For the next page, use `after_event_index` and pass there the index from the last event from the previous page.
///
/// **Limitations**
/// * If the history does not match with the balances from RPC, we do not support the history (see Trusted/Inconsistent Contracts section below).
/// * We are in the process of supporting Multi Token history.
///
/// **Trusted/Inconsistent Contracts**
/// The contracts produce [the balance changing events](https://nomicon.io/Standards/Tokens/FungibleToken/Event).
/// The contract is marked as inconsistent if its events do not match with the balance taken from RPC.
/// The contract is marked as trusted otherwise.
/// More info could be found [here](https://github.com/near/near-indexer-events#my-contract-produces-eventstheres-a-custom-legacy-logic-for-my-contract-but-the-enhanced-api-still-ignores-me-why).
pub async fn get_coin_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    pool_balances: web::Data<db_helpers::DBWrapper>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::HistoryRequest>,
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native history, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }

    let pagination = modules::checked_get_pagination_params(&pagination_params).await?;
    let block = db_helpers::checked_get_block_from_pagination(&pool, &pagination).await?;
    // we don't need to check whether account exists. If not, we can just return the empty history

    Ok(Json(schemas::HistoryResponse {
        history: data_provider::get_coin_history(
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

#[api_v2_operation(tags(Coins))]
/// Get FT contract metadata
///
/// This endpoint returns the metadata for a given FT contract and `timestamp`/`block_height`.
///
/// **Limitations**
/// * For now, we support only FT contracts that implement the Events NEP standard.
///   We are working on a solution to support other FT contracts, including `wrap.near` and bridged tokens.
pub async fn get_ft_contract_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::ContractMetadataRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::FtContractMetadataResponse>> {
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;

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
