use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
};

use crate::{db_helpers, modules, types};

use super::schemas;

#[api_v2_operation(tags(NFT))]
/// Get user's NFT collection overview
///
/// For the given `account_id` and `timestamp` or `block_height`, this endpoint returns
/// the number of NFTs grouped by `contract_id`, together with the corresponding NFT contract metadata.
/// The NFT contract will be present in the response if the `account_id` has at least one NFT there.
///
/// **Note:** `block_timestamp_nanos` helps you choose a moment in time, fixing the blockchain state at that time.
///
/// **Limitations**
/// * We currently provide the most recent 100 items.
///   Full-featured pagination will be provided in later phases.
pub async fn get_nft_collection_overview(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::NftCountsRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    limit_params: web::Query<types::query_params::LimitParams>,
) -> crate::Result<Json<schemas::NftCountsResponse>> {
    let limit = types::query_params::checked_get_limit(limit_params.limit)?;
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    Ok(Json(schemas::NftCountsResponse {
        // TODO PHASE 2 We can data_provider metadata in the DB and update once in 10 minutes
        nft_counts: super::data_provider::get_nfts_count(
            &pool,
            &rpc_client,
            &block,
            &request.account_id.0,
            limit,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation(tags(NFT))]
/// Get user's NFT collection by contract
///
/// This endpoint returns the list of NFTs with full details for the given `account_id`, NFT `contract_id`, `timestamp`/`block_height`.
/// You can use the `token_id` from this response and then request the NFT history for that token.
///
/// **Limitations**
/// * We currently provide the most recent 100 items.
///   Full-featured pagination will be provided in later phases.
pub async fn get_nft_collection_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::NftCollectionRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
    limit_params: web::Query<types::query_params::LimitParams>,
) -> crate::Result<Json<schemas::NftsResponse>> {
    let limit = types::query_params::checked_get_limit(limit_params.limit)?;
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;
    modules::check_account_exists(&rpc_client, &request.account_id.0, block.height).await?;

    Ok(Json(schemas::NftsResponse {
        nfts: super::data_provider::get_nfts_by_contract(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.account_id.0.clone(),
            block.height,
            limit,
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

#[api_v2_operation(tags(NFT))]
/// Get NFT
///
/// This endpoint returns detailed information on the NFT
/// for the given `token_id`, NFT `contract_id`, `timestamp`/`block_height`.
pub async fn get_nft(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::NftRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::NftResponse>> {
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;

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

#[api_v2_operation(tags(NFT))]
/// Get NFT history
///
/// This endpoint returns the transaction history for the given NFT and `timestamp`/`block_height`.
/// **Note:** The result is centered around the history of the specific NFT and will return list of its passing owners and metadata.
///
/// **Limitations**
/// * For now, we only support NFT contracts that implement the Events NEP standard.
/// * We currently provide the most recent 100 items.
///   Full-featured pagination will be provided in later phases.
pub async fn get_nft_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::NftRequest>,
    limit_params: web::Query<types::query_params::LimitParams>,
) -> crate::Result<Json<schemas::HistoryResponse>> {
    let limit = types::query_params::checked_get_limit(limit_params.limit)?;
    let block = db_helpers::get_last_block(&pool).await?;

    Ok(Json(schemas::HistoryResponse {
        history: super::data_provider::get_nft_history(
            &pool,
            &request.contract_account_id.0,
            &request.token_id,
            &block,
            limit,
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

#[api_v2_operation(tags(NFT))]
/// Get NFT contract metadata
///
/// This endpoint returns the metadata for a given NFT contract and `timestamp`/`block_height`.
/// **Note:** This is contract-wide metadata. Each NFT also has its own metadata.
pub async fn get_nft_contract_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    _: crate::types::pagoda_api_key::PagodaApiKey,
    request: actix_web_validator::Path<schemas::MetadataRequest>,
    block_params: web::Query<types::query_params::BlockParams>,
) -> crate::Result<Json<schemas::MetadataResponse>> {
    let block = db_helpers::checked_get_block(&pool, &block_params).await?;

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
