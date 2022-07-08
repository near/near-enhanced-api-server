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

// to sum up
// re-check docs here
// re-check todos at this file (other files have already re-checked)

// todo re-check tests coverage, I think we need more
// todo write creds to the doc
// todo MT
// todo statistics collection
// todo add overflow docs everywhere
// todo page + limit. By timestamp/height
// todo think about pagination/sorting, create the doc with available options
// todo decide tables architecture (one/many) for ft/mt, make the list with pros and cons
// todo design the idea of endpoints by symbols
// todo debug vlad.near nfts
// todo try to add more tests
const DEFAULT_PAGE_LIMIT: u32 = 20;
const MAX_PAGE_LIMIT: u32 = 100;

#[api_v2_operation]
/// Get user's NEAR balance
///
/// This endpoint returns the NEAR balance of the given account_id
/// for the given timestamp/block_height.
pub async fn near_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NearBalanceResponse>> {
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(
        api::near_balance(&pool, &block, &request.account_id.0).await?,
    ))
}

#[api_v2_operation]
/// Get user's coin balances
///
/// This endpoint returns all the countable coin balances (including NEAR) of the given account_id,
/// for the given timestamp/block_height.
/// Pagination will be provided later.
pub async fn coin_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    // TODO PHASE 1 discuss whether we want to leave block_params here. I feel we need this.
    // the request GET ...?block_height=...&no_updates_after_block_height=... has sense
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::BalancesPaginationParams>,
) -> api_models::Result<Json<api_models::BalancesResponse>> {
    // TODO PHASE 2 pagination by index
    utils::check_limit(pagination_params.limit)?;
    let mut pagination: types::CoinBalancesPagination = pagination_params.0.into();
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    // ordering: near, then fts by contract_id, then mts by contract_id and symbol
    let mut balances: Vec<api_models::Coin> = vec![];
    // if pagination.last_standard.is_none() {
    balances.push(
        api::near_balance(&pool, &block, &request.account_id.0)
            .await?
            .into(),
    );
    pagination.limit -= 1;
    // }
    if pagination.limit > 0
    // && (pagination.last_standard.is_none()
    //     || pagination.last_standard == Some("nep141".to_string()))
    {
        // todo put the constants in one place
        let fts = &mut api::ft_balance(
            &pool,
            &rpc_client,
            &block,
            &request.account_id.0,
            &pagination,
        )
        .await?;
        balances.append(fts);
        pagination.limit -= fts.length() as u32;
    }
    // todo remember here could be mt
    // if pagination.limit > 0 {
    //     //...
    // }

    Ok(Json(api_models::BalancesResponse {
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
pub async fn balance_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceByContractRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::BalancesResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native balance, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    // todo remember here could be mt
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;
    utils::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let mut balances: Vec<api_models::Coin> = vec![];
    // todo how it's better to check does such ft contract exist? User could ask about MT contract. Or about broken contract, or not existing contract.
    balances.push(
        api::ft_balance_for_contract(
            &rpc_client,
            &block,
            &request.contract_account_id.0,
            &request.account_id.0,
        )
        .await?,
    );

    Ok(Json(api_models::BalancesResponse {
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
/// NFT contract is presented in the list if the account_id has at least one NFT there.
/// Sorted by the change order: recent go first.
/// Be careful with the 2 timestamp that you can provide here.
/// block_timestamp_nanos helps you to choose the moment of time, we fix the blockchain state at that time.
/// with_no_updates_after_timestamp_nanos helps you to paginate the data that we've fixed before
pub async fn get_user_nfts_overview(
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
        // todo we still need rpc_client only for metadata. We can store this in the DB and update once in 10 minutes
        nft_collection_overview: api::nft_count(
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

// TODO MAJOR ISSUE we should fix the DB and ignore failed receipts/transactions while collecting the data about FTs/NFTs
// after that, we need to re-check all this stuff again, maybe it's not the only issue here
// By the way, we need to check who should fix that (I mean, sometimes the contract should log the opposite movement of the token)
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
pub async fn dev_get_user_nfts_overview(
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
        nft_collection_overview: api::dev_nft_count(
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
/// This endpoint returns the list of NFTs with token metadata
/// for the given account_id, NFT contract_id, timestamp/block_height.
/// You can copy the token_id from this response and ask for NFT history.
/// Pagination will be provided later.
pub async fn get_user_nfts_by_contract(
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
        nft_collection: rpc_api::get_nfts(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.account_id.0.clone(),
            block.height,
            pagination_params.limit.unwrap_or(DEFAULT_PAGE_LIMIT),
        )
        .await?,
        contract_metadata: rpc_api::get_nft_general_metadata(
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
/// This endpoint returns the NFT details
/// for the given token_id, NFT contract_id, timestamp/block_height.
pub async fn nft_item_details(
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
        contract_metadata: rpc_api::get_nft_general_metadata(
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
/// Sorted in a historical descending order.
pub async fn near_history(
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
        near_history: api::near_history(&pool_balances.pool, &request.account_id, &pagination)
            .await?,
        near_metadata: api::near_metadata(),
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
pub async fn coin_history(
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

    // todo remember here could be mt
    // todo pages

    Ok(Json(api_models::CoinHistoryResponse {
        coin_history: api::coin_history(
            &pool,
            &rpc_client,
            &request.contract_account_id.0,
            &request.account_id.0,
            &pagination,
        )
        .await?,
        contract_metadata: rpc_api::get_coin_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            pagination.block_height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(pagination.block_timestamp),
        block_height: types::U64::from(pagination.block_height),
    }))
}

#[api_v2_operation]
/// Get NFT history
///
/// This endpoint returns the history of operations for the given NFT and timestamp/block_height.
/// Keep in mind, it does not related to a concrete account_id; the whole history is shown.
/// Sorted in a historical descending order.
pub async fn nft_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::NftRequest>,
    pagination_params: web::Query<api_models::HistoryPaginationParams>,
) -> api_models::Result<Json<api_models::NftHistoryResponse>> {
    let block = api::get_last_block(&pool).await?;
    let pagination =
        utils::check_and_get_history_pagination_params(&pool, &pagination_params).await?;

    Ok(Json(api_models::NftHistoryResponse {
        token_history: api::nft_history(
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
        contract_metadata: rpc_api::get_nft_general_metadata(
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
pub async fn ft_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::ContractMetadataRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::FtContractMetadataResponse>> {
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(api_models::FtContractMetadataResponse {
        contract_metadata: rpc_api::get_ft_metadata(
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
pub async fn nft_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::ContractMetadataRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NftContractMetadataResponse>> {
    utils::check_block_params(&block_params)?;
    let block = api::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(api_models::NftContractMetadataResponse {
        contract_metadata: rpc_api::get_nft_general_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}
