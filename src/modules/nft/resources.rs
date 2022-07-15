use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use crate::{db_helpers, modules, types};

use super::schemas;

#[api_v2_operation]
/// Get user's NFT collection overview
///
/// For the given account_id and timestamp/block_height, this endpoint returns
/// the number of NFTs grouped by contract_id, together with the corresponding NFT contract metadata.
/// NFT contract is presented if the account_id has at least one NFT there.
///
/// `block_timestamp_nanos` helps you to choose the moment of time, we fix the blockchain state at that time.
///
/// **Limitations**
/// * We provide only up to 100 items, where recently updated data goes first.
///   Full-featured pagination will be provided later.
pub async fn get_nft_collection_overview(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::NftCountsRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::NftCountsResponse>> {
    types::query_params::check_limit(pagination_params.limit)?;
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
    modules::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(schemas::NftCountsResponse {
        // TODO PHASE 2 We can data_provider metadata in the DB and update once in 10 minutes
        nft_counts: super::data_provider::get_nfts_count(
            &pool,
            &rpc_client,
            &block,
            &request.account_id.0,
            pagination_params.0,
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
/// **Limitations**
/// * We provide only up to 100 items.
///   Full-featured pagination will be provided later.
pub async fn get_nft_collection_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::NftCollectionRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    pagination_params: web::Query<types::query_params::PaginationParams>,
) -> crate::Result<Json<schemas::NftCollectionResponse>> {
    types::query_params::check_limit(pagination_params.limit)?;
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;
    modules::check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;
    let pagination = types::query_params::Pagination::from(pagination_params.0);

    Ok(Json(schemas::NftCollectionResponse {
        nft_collection: super::data_provider::get_nfts_by_contract(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.account_id.0.clone(),
            block.height,
            pagination.limit,
        )
        .await?,
        contract_metadata: super::data_provider::get_nft_contract_metadata(
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
/// Get NFT
///
/// This endpoint returns the NFT detailed information
/// for the given token_id, NFT contract_id, timestamp/block_height.
pub async fn get_nft(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::NftRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::NftResponse>> {
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(schemas::NftResponse {
        nft: super::data_provider::get_nft(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.token_id.clone(),
            block.height,
        )
        .await?,
        contract_metadata: super::data_provider::get_nft_contract_metadata(
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
/// Get NFT history
///
/// This endpoint returns the history of operations for the given NFT and timestamp/block_height.
/// Keep in mind, it does not related to a concrete account_id; the whole history is shown.
///
/// **Limitations**
/// * For now, we support only NFT contracts which implement Events NEP.
/// * We provide only up to 100 items, where recent updates go first.
///   Full-featured pagination will be provided later.
pub async fn get_nft_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<schemas::NftRequest>,
    pagination_params: web::Query<types::query_params::HistoryPaginationParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    let block = db_helpers::get_last_block(&pool).await?;
    let pagination =
        modules::check_and_get_history_pagination_params(&pool, pagination_params.0).await?;

    Ok(Json(schemas::HistoryResponse {
        history: super::data_provider::get_nft_history(
            &pool,
            &request.contract_account_id.0,
            &request.token_id,
            &pagination,
        )
        .await?,
        nft: super::data_provider::get_nft(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.token_id.clone(),
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
    request: web::Path<schemas::MetadataRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::MetadataResponse>> {
    types::query_params::check_block_params(&block_params)?;
    let block = db_helpers::get_block_from_params(&pool, &block_params).await?;

    Ok(Json(schemas::MetadataResponse {
        metadata: super::data_provider::get_nft_contract_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}
