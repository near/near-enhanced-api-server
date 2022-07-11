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

// TODO PHASE 1 I want to add stats collection. Do we have time for that?
// TODO PHASE 1 I had the note "add overflow docs everywhere". Now I feel we don't need that
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
/// Pagination will be provided later.
pub async fn get_coin_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    // TODO PHASE 1 discuss whether we want to leave block_params here. I feel we need this.
    // the request GET ...?block_height=...&no_updates_after_block_height=... has sense
    block_params: web::Query<api_models::BlockParams>,
    // TODO PHASE 2 pagination by index (recently updated go first)
    pagination_params: web::Query<api_models::BalancesPaginationParams>,
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
pub async fn get_balances_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceByContractRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::CoinBalancesResponse>> {
    // TODO PHASE 1 do we want to redirect to NEAR balance? The format of the response will change
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
/// Sorted by the change order: recent go first. Pagination will be provided later.
///
/// `block_timestamp_nanos` helps you to choose the moment of time, we fix the blockchain state at that time.
pub async fn get_nft_collection_overview(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    // TODO PHASE 1 discuss whether we want to leave block_params here. I feel we need this.
    // the request GET ...?block_height=...&no_updates_after_block_height=... has sense
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::BalancesPaginationParams>,
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

// TODO PHASE 1 MAJOR ISSUE we should fix the DB and ignore failed receipts/transactions while collecting the data about FTs/NFTs
// after that, we need to re-check all this stuff again, maybe it's not the only issue here
// By the way, we need to check who should fix that (I mean, sometimes it's the the contract who should log the opposite movement of the token)
#[api_v2_operation]
/// Issues with the contracts: thebullishbulls.near x.paras.near
///
/// We can see here that token 1349 was issued to 3 different users
/// select * from assets__non_fungible_token_events
/// where token_id = '1349' and emitted_by_contract_account_id = 'thebullishbulls.near'
/// order by emitted_at_block_timestamp desc;
///
/// receipts about 2 other users actually failed:
/// select * from receipts join execution_outcomes on receipts.receipt_id = execution_outcomes.receipt_id
/// where execution_outcomes.receipt_id in ('7P36s12WJQDnqdwyLZRRWoApvXnTuB8JnuFGoyWgpm49', 'HCM87NB9wXw3P3YoCf6u4kc4G45DsoyjV5Robanrcstt');
///
/// Here, we can see that user should still have 8 tokens (11 mints - 3 transfers = 8 should be still here)
/// But the contract says they have nothing
/// select * from assets__non_fungible_token_events
/// where (token_new_owner_account_id = 'kbneoburner3.near' or token_old_owner_account_id = 'kbneoburner3.near')
/// and emitted_by_contract_account_id = 'thebullishbulls.near'
/// order by emitted_at_block_timestamp desc;
///
/// Same issues with x.paras.near
pub async fn get_nft_collection_overview_dev(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::BalancesPaginationParams>,
) -> api_models::Result<Json<api_models::NftCollectionOverviewResponse>> {
    utils::check_limit(pagination_params.limit)?;
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(api_models::NftCollectionOverviewResponse {
        nft_collection_overview: api::get_nft_count_dev(
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
/// Pagination will be provided later.
pub async fn get_nft_collection_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceByContractRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::BalancesPaginationParams>,
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
/// Sorted in a historical descending order. Pagination will be provided later.
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
        near_history: api::get_near_history(&pool_balances.pool, &request.account_id, &pagination)
            .await?,
        near_metadata: api::get_near_metadata(),
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
/// Get user's coin history by contract
///
/// This endpoint returns the history of coin operations (FT, other standards)
/// for the given account_id, contract_id, timestamp/block_height.
/// Sorted in a historical descending order.
/// Pagination will be provided later.
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
/// Sorted in a historical descending order. Pagination will be provided later.
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
        token_metadata: rpc_api::get_nft_metadata(
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
