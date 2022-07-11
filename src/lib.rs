use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};
use sqlx::types::BigDecimal;
use validator::HasLen;

mod api;
mod api_models;
pub mod config;
mod db_models;
pub mod errors;
mod rpc_api;
pub mod types;
mod utils;

const DEFAULT_PAGE_LIMIT: u32 = 20;
const MAX_PAGE_LIMIT: u32 = 100;

#[api_v2_operation]
/// Get user's NEAR balance
///
/// This endpoint returns the NEAR balance of the given account_id
/// for the given timestamp/block_height.
pub async fn get_near_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NearBalanceResponse>> {
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(
        api::get_near_balance(&pool, &block, &request.account_id.0).await?,
    ))
}

#[api_v2_operation]
/// Get user's coin balances
///
/// This endpoint returns all the countable coin balances (including NEAR, FTs, later will add MTs)
/// of the given account_id, for the given timestamp/block_height.
///
/// ** Limitations **
///
/// * For now, we support only the balance for NEAR and FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
/// * We are in the process of supporting Multi Token balances.
/// * We provide only up to 100 items, where recently updated data goes first.
///   Full-featured pagination will be provided later.
pub async fn get_coin_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
    // TODO PHASE 2 pagination by index (recently updated go first)
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::CoinBalancesResponse>> {
    utils::check_limit(pagination_params.limit)?;
    let mut pagination: types::CoinBalancesPagination = pagination_params.0.into();
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let mut balances: Vec<api_models::Coin> = vec![];
    balances.push(
        api::get_near_balance(&pool, &block, &request.account_id.0)
            .await?
            .into(),
    );
    pagination.limit -= 1;

    if pagination.limit > 0 {
        let ft_balances = &mut api::get_ft_balances(
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

    Ok(Json(api_models::CoinBalancesResponse {
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
/// ** Limitations **
///
/// * For now, we support only the balance for FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
/// * We are in the process of supporting Multi Token balances.
pub async fn get_balances_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceByContractRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::CoinBalancesResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native balance, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let mut balances: Vec<api_models::Coin> = vec![];
    // When we add MT here, we need to filter out the responses like "standard not implemented"
    balances.push(
        api::get_ft_balance_for_contract(
            &rpc_client,
            &block,
            &request.contract_account_id.0,
            &request.account_id.0,
        )
        .await?,
    );

    Ok(Json(api_models::CoinBalancesResponse {
        balances,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get user's NFT collection overview
///
/// For the given account_id and timestamp/block_height, this endpoint returns
/// the number of NFTs grouped by contract_id, together with the corresponding NFT contract metadata.
/// NFT contract is presented if the account_id has at least one NFT there.
///
/// `block_timestamp_nanos` helps you to choose the moment of time, we fix the blockchain state at that time.
///
/// ** Limitations **
///
/// * We provide only up to 100 items, where recently updated data goes first.
///   Full-featured pagination will be provided later.
pub async fn get_nft_collection_overview(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::NftCollectionOverviewResponse>> {
    utils::check_limit(pagination_params.limit)?;
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(api_models::NftCollectionOverviewResponse {
        // TODO PHASE 2 We can store metadata in the DB and update once in 10 minutes
        nft_collection_overview: api::get_nft_count(
            &pool,
            &rpc_client,
            &block,
            &request.account_id.0,
            &pagination_params,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get user's NFT collection by contract
///
/// This endpoint returns the list of NFTs, each of them contains all the detailed NFT information,
/// for the given account_id, NFT contract_id, timestamp/block_height.
/// You can copy the token_id from this response and then ask for NFT history.
///
/// ** Limitations **
///
/// * We provide only up to 100 items.
///   Full-featured pagination will be provided later.
pub async fn get_nft_collection_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceByContractRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::NftCollectionByContractResponse>> {
    utils::check_limit(pagination_params.limit)?;
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(api_models::NftCollectionByContractResponse {
        nft_collection: rpc_api::get_nft_collection(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.account_id.0.clone(),
            block.height,
            pagination_params.limit.unwrap_or(DEFAULT_PAGE_LIMIT),
        )
        .await?,
        contract_metadata: rpc_api::get_nft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get NFT details
///
/// This endpoint returns the NFT detailed information
/// for the given token_id, NFT contract_id, timestamp/block_height.
pub async fn get_nft_item_details(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::NftRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NftResponse>> {
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(api_models::NftResponse {
        nft: rpc_api::get_nft_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.token_id.clone(),
            block.height,
        )
        .await?,
        contract_metadata: rpc_api::get_nft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
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
/// ** Limitations **
///
/// * We provide only up to 100 items, where recent updates go first.
///   Full-featured pagination will be provided later.
pub async fn get_near_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    pool_balances: web::Data<types::DBWrapper>,
    request: web::Path<api_models::BalanceRequest>,
    pagination_params: web::Query<api_models::HistoryPaginationParams>,
) -> api_models::Result<Json<api_models::NearHistoryResponse>> {
    let block = api::get_last_block(&pool).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;
    let pagination =
        utils::check_and_get_history_pagination_params(&pool, &pagination_params).await?;

    Ok(Json(api_models::NearHistoryResponse {
        coin_history: api::get_near_history(&pool_balances.pool, &request.account_id, &pagination)
            .await?,
        contract_metadata: api::get_near_metadata(),
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
/// ** Limitations **
///
/// * For now, we support only FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
/// * We are in the process of supporting Multi Token history.
/// * We provide only up to 100 items, where recent updates go first.
///   Full-featured pagination will be provided later.
pub async fn get_coin_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceHistoryRequest>,
    pagination_params: web::Query<api_models::HistoryPaginationParams>,
) -> api_models::Result<Json<api_models::CoinHistoryResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native history, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    let pagination =
        utils::check_and_get_history_pagination_params(&pool, &pagination_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, pagination.block_timestamp).await?;

    Ok(Json(api_models::CoinHistoryResponse {
        coin_history: api::get_ft_history(
            &pool,
            &rpc_client,
            &request.contract_account_id.0,
            &request.account_id.0,
            &pagination,
        )
        .await?,
        contract_metadata: rpc_api::get_ft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            pagination.block_height,
        )
        .await?
        .into(),
        block_timestamp_nanos: types::U64::from(pagination.block_timestamp),
        block_height: types::U64::from(pagination.block_height),
    }))
}

#[api_v2_operation]
/// Get NFT history
///
/// This endpoint returns the history of operations for the given NFT and timestamp/block_height.
/// Keep in mind, it does not related to a concrete account_id; the whole history is shown.
///
/// ** Limitations **
///
/// * For now, we support only NFT contracts which implement Events NEP.
/// * We provide only up to 100 items, where recent updates go first.
///   Full-featured pagination will be provided later.
pub async fn get_nft_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::NftRequest>,
    pagination_params: web::Query<api_models::HistoryPaginationParams>,
) -> api_models::Result<Json<api_models::NftHistoryResponse>> {
    let block = api::get_last_block(&pool).await?;
    let pagination =
        utils::check_and_get_history_pagination_params(&pool, &pagination_params).await?;

    Ok(Json(api_models::NftHistoryResponse {
        token_history: api::get_nft_history(
            &pool,
            &request.contract_account_id.0,
            &request.token_id,
            &pagination,
        )
        .await?,
        token: rpc_api::get_nft_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.token_id.clone(),
            block.height,
        )
        .await?,
        contract_metadata: rpc_api::get_nft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get FT contract metadata
///
/// This endpoint returns the metadata for given FT contract and timestamp/block_height.
///
/// ** Limitations **
///
/// * For now, we support only FT contracts which implement Events NEP.
///   We work on the solution to support the other FT contracts, including `wrap.near` and bridged tokens.
pub async fn get_ft_contract_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::ContractMetadataRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::FtContractMetadataResponse>> {
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(api_models::FtContractMetadataResponse {
        contract_metadata: rpc_api::get_ft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get NFT contract metadata
///
/// This endpoint returns the metadata for given NFT contract and timestamp/block_height.
/// Keep in mind, this is contract-wide metadata. Each NFT also has its own metadata.
pub async fn get_nft_contract_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::ContractMetadataRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NftContractMetadataResponse>> {
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(api_models::NftContractMetadataResponse {
        contract_metadata: rpc_api::get_nft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}
