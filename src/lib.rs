use crate::utils::add_items;
use actix_cors::Cors;
use actix_web::{App, HttpServer, ResponseError};
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    OpenApiExt,
};
use sqlx::types::BigDecimal;
use validator::HasLen;

mod api;
mod api_models;
pub mod config;
mod db_models;
mod errors;
mod rpc_calls;
mod types;
mod utils;

// todo write creds to the doc
// todo MT
// todo statistics collection
// todo learn how to return page/limit info in the headers response
const MAX_PAGE_LIMIT: u32 = 100;

#[api_v2_operation]
/// Get the user's balance
///
/// This endpoint returns the balance of the given account_id,
/// for the specified token_contract_id | near.
async fn native_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NearBalanceResponse>> {
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(
        api::native_balance(&pool, &block, &request.account_id.0).await?,
    ))
}

#[api_v2_operation]
/// Get the user's balance
///
/// This endpoint returns the balance of the given account_id,
/// for the specified token_contract_id | near.
async fn coin_balances(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::BalancesResponse>> {
    let mut pagination = check_and_get_pagination(&pagination_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    // ordering: near, then fts by contract_id, then mts by contract_id and symbol
    let mut balances: Vec<api_models::Coin> = vec![];
    if pagination.offset == 0 {
        balances.push(
            api::native_balance(&pool, &block, &request.account_id.0)
                .await?
                .into(),
        );
        add_items(&mut pagination, 1)?;
    }
    if pagination.limit > 0 {
        let fts = &mut api::ft_balance(
            &pool,
            &rpc_client,
            &block,
            &request.account_id.0,
            &pagination,
        )
        .await?;
        balances.append(fts);
        add_items(&mut pagination, fts.length() as u32)?;
    }
    // todo remember here could be mt
    // if pagination.limit > 0 {
    //     //...
    //     add_items(&mut pagination, ...)?;
    // }

    Ok(Json(api_models::BalancesResponse {
        balances,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
async fn balance_by_contract(
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
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let mut balances: Vec<api_models::Coin> = vec![];
    if let Some(ft) = api::ft_balance_for_contract(
        &rpc_client,
        &block,
        &request.contract_account_id.0,
        &request.account_id.0,
    )
    .await?
    {
        balances.push(ft);
    }

    Ok(Json(api_models::BalancesResponse {
        balances,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
async fn nft_balance_overview(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::NftCountResponse>> {
    let pagination = check_and_get_pagination(&pagination_params)?;
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    // todo pages
    Ok(Json(api_models::NftCountResponse {
        nft_count: api::nft_count(&pool, &rpc_client, &block, &request.account_id.0).await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

// todo re-check the answer, it's strange. Owner account id does not match
#[api_v2_operation]
async fn nft_balance_detailed(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceByContractRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::NftBalanceResponse>> {
    let pagination = check_and_get_pagination(&pagination_params)?;
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    // todo pages
    Ok(Json(api_models::NftBalanceResponse {
        nfts: api::nft_by_contract(
            &pool,
            &rpc_client,
            &block,
            &request.contract_account_id.0,
            &request.account_id.0,
        )
        .await?,
        contract_metadata: rpc_calls::get_nft_general_metadata(
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
async fn nft_item_details(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::NftItemRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NftItemResponse>> {
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    Ok(Json(api_models::NftItemResponse {
        nft: rpc_calls::get_nft_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            request.token_id.clone(),
            block.height,
        )
        .await?,
        contract_metadata: rpc_calls::get_nft_general_metadata(
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
async fn native_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::NftItemRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::NearHistoryResponse>> {
    todo!("not implemented yet");
}

#[api_v2_operation]
async fn coin_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::BalanceHistoryRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::HistoryResponse>> {
    if request.contract_account_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native history, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    let pagination = check_and_get_pagination(&pagination_params)?;
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    // todo remember here could be mt
    // todo pages
    let history = api::ft_history(
        &pool,
        &rpc_client,
        &block,
        &request.contract_account_id.0,
        &request.account_id.0,
    )
    .await?;

    Ok(Json(api_models::HistoryResponse {
        history,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

#[api_v2_operation]
async fn nft_history(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::NftItemRequest>,
    block_params: web::Query<api_models::BlockParams>,
    pagination_params: web::Query<api_models::PaginationParams>,
) -> api_models::Result<Json<api_models::NftHistoryResponse>> {
    todo!("not implemented yet");
}

#[api_v2_operation]
async fn ft_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::ContractMetadataRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::FtMetadataResponse>> {
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;

    Ok(Json(api_models::FtMetadataResponse {
        metadata: rpc_calls::get_ft_metadata(
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
async fn nft_metadata(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::ContractMetadataRequest>,
    block_params: web::Query<api_models::BlockParams>,
) -> api_models::Result<Json<api_models::NftMetadataResponse>> {
    check_block_params(&block_params)?;
    let block = get_block_from_params(&pool, &block_params).await?;

    Ok(Json(api_models::NftMetadataResponse {
        metadata: rpc_calls::get_nft_general_metadata(
            &rpc_client,
            request.contract_account_id.0.clone(),
            block.height,
        )
        .await?,
        block_timestamp_nanos: types::U64::from(block.timestamp),
        block_height: types::U64::from(block.height),
    }))
}

fn check_block_params(params: &web::Query<api_models::BlockParams>) -> api_models::Result<()> {
    if params.block_height.is_some() && params.block_timestamp_nanos.is_some() {
        Err(errors::ErrorKind::InvalidInput(
            "Both block_height and block_timestamp_nanos found. Please provide only one of values"
                .to_string(),
        )
        .into())
    } else {
        Ok(())
    }
}

fn check_and_get_pagination(
    params: &web::Query<api_models::PaginationParams>,
) -> api_models::Result<types::Pagination> {
    if let Some(limit) = params.limit {
        if limit > MAX_PAGE_LIMIT || limit == 0 {
            return Err(errors::ErrorKind::InvalidInput(format!(
                "Limit should be in range [1, {}]",
                MAX_PAGE_LIMIT
            ))
            .into());
        }
    }
    Ok(types::Pagination {
        offset: params.offset.unwrap_or(0),
        limit: params.limit.unwrap_or(20),
    })
}

// todo do we need check_contract_exists? (now we will just fail when we make the call to rpc)
async fn check_account_exists(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> api_models::Result<()> {
    if !api::account_exists(pool, account_id, block_timestamp).await? {
        Err(errors::ErrorKind::InvalidInput(format!(
            "account_id {} does not exist at block_timestamp {}",
            account_id, block_timestamp
        ))
        .into())
    } else {
        Ok(())
    }
}

const RETRY_COUNT: usize = 1;

async fn get_block_from_params(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
    params: &web::Query<api_models::BlockParams>,
) -> api_models::Result<types::Block> {
    if let Some(block_height) = params.block_height {
        match utils::select_retry_or_panic::<db_models::Block>(
            pool,
            "SELECT block_height, block_timestamp FROM blocks WHERE block_height = $1::numeric(20, 0)",
            &[block_height.0.to_string()],
            RETRY_COUNT,
        )
        .await?
        .first() {
            None => Err(errors::ErrorKind::DBError(format!("block_height {} is not found", block_height.0)).into()),
            Some(block) => Ok(types::Block::try_from(block)?)
        }
    } else if let Some(block_timestamp) = params.block_timestamp_nanos {
        match utils::select_retry_or_panic::<db_models::Block>(
            pool,
            r"SELECT block_height, block_timestamp
              FROM blocks
              WHERE block_timestamp <= $1::numeric(20, 0)
              ORDER BY block_timestamp DESC
              LIMIT 1",
            &[block_timestamp.0.to_string()],
            RETRY_COUNT,
        )
        .await?
        .first()
        {
            None => get_first_block(pool).await,
            Some(block) => Ok(types::Block::try_from(block)?),
        }
    } else {
        get_last_block(pool).await
    }
}

async fn get_first_block(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
) -> api_models::Result<types::Block> {
    match utils::select_retry_or_panic::<db_models::Block>(
        pool,
        r"SELECT block_height, block_timestamp
          FROM blocks
          ORDER BY block_timestamp
          LIMIT 1",
        &[],
        RETRY_COUNT,
    )
    .await?
    .first()
    {
        None => Err(errors::ErrorKind::DBError("blocks table is empty".to_string()).into()),
        Some(block) => Ok(types::Block::try_from(block)?),
    }
}

async fn get_last_block(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
) -> api_models::Result<types::Block> {
    match utils::select_retry_or_panic::<db_models::Block>(
        pool,
        r"SELECT block_height, block_timestamp
          FROM blocks
          ORDER BY block_timestamp DESC
          LIMIT 1",
        &[],
        RETRY_COUNT,
    )
    .await?
    .first()
    {
        None => Err(errors::ErrorKind::DBError("blocks table is empty".to_string()).into()),
        Some(block) => Ok(types::Block::try_from(block)?),
    }
}

fn get_cors(cors_allowed_origins: &[String]) -> Cors {
    let mut cors = Cors::permissive();
    if cors_allowed_origins != ["*".to_string()] {
        for origin in cors_allowed_origins {
            cors = cors.allowed_origin(origin);
        }
    }
    cors.allowed_methods(vec!["GET"])
        .allowed_headers(vec![
            actix_web::http::header::AUTHORIZATION,
            actix_web::http::header::ACCEPT,
        ])
        .allowed_header(actix_web::http::header::CONTENT_TYPE)
        .max_age(3600)
}

pub fn start(
    config: config::Config,
    pool: sqlx::Pool<sqlx::Postgres>,
    rpc_client: near_jsonrpc_client::JsonRpcClient,
) -> actix_web::dev::ServerHandle {
    let config::Config {
        addr,
        cors_allowed_origins,
        limits,
    } = config;
    let addr_clone = addr.clone();
    let server = HttpServer::new(move || {
        let json_config = web::JsonConfig::default()
            .limit(limits.input_payload_max_size)
            .error_handler(|err, _req| {
                let error_message = err.to_string();
                actix_web::error::InternalError::from_response(
                    err,
                    errors::Error::from_error_kind(errors::ErrorKind::InvalidInput(error_message))
                        .error_response(),
                )
                .into()
            });

        let mut spec = paperclip::v2::models::DefaultApiRaw::default();
        spec.host = Some(addr_clone.clone());
        spec.base_path = Some("/".to_string());
        spec.tags = vec![
            paperclip::v2::models::Tag {
                name: "Accounts".to_string(),
                description: Some("Most common actions with accounts in NEAR".to_string()),
                external_docs: None,
            },
            paperclip::v2::models::Tag {
                name: "Standards".to_string(),
                description: Some(
                    "Manipulate with NEAR Enhancement Proposal (NEP) Standards".to_string(),
                ),
                external_docs: None,
            },
        ];
        spec.info = paperclip::v2::models::Info {
            version: "0.1".into(),
            title: "NEAR Enchanced API (by Pagoda Inc)".into(),
            ..Default::default()
        };

        App::new()
            .app_data(json_config)
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(rpc_client.clone()))
            .wrap(get_cors(&cors_allowed_origins))
            .wrap_api_with_spec(spec)
            .service(
                web::resource("/accounts/{account_id}/coins/NEAR")
                    .route(web::get().to(native_balance)),
            )
            .service(
                web::resource("/accounts/{account_id}/coins").route(web::get().to(coin_balances)),
            )
            .service(
                web::resource("/accounts/{account_id}/coins/{contract_account_id}")
                    .route(web::get().to(balance_by_contract)),
            )
            .service(
                web::resource("/accounts/{account_id}/collectibles")
                    .route(web::get().to(nft_balance_overview)),
            )
            .service(
                web::resource("/accounts/{account_id}/collectibles/{contract_account_id}")
                    .route(web::get().to(nft_balance_detailed)),
            )
            .service(
                web::resource(
                    "/accounts/{account_id}/collectibles/{contract_account_id}/{token_id}",
                )
                .route(web::get().to(nft_item_details)),
            )
            .service(
                web::resource("/accounts/{account_id}/coins/NEAR/history")
                    .route(web::get().to(native_history)),
            )
            .service(
                web::resource("/accounts/{account_id}/coins/{contract_account_id}/history")
                    .route(web::get().to(coin_history)),
            )
            .service(
                web::resource(
                    "/accounts/{account_id}/collectibles/{contract_account_id}/{token_id}/history",
                )
                .route(web::get().to(nft_history)),
            )
            .service(
                web::resource("/nep141/metadata/{contract_account_id}")
                    .route(web::get().to(ft_metadata)),
            )
            .service(
                web::resource("/nep171/metadata/{contract_account_id}")
                    .route(web::get().to(nft_metadata)),
            )
            .with_json_spec_at("/api/spec/v2.json")
            .with_json_spec_v3_at("/api/spec/v3.json")
            .build()
    })
    .bind(addr)
    .unwrap()
    .shutdown_timeout(5)
    .disable_signals()
    .run();

    let handle = server.handle();

    tokio::spawn(server);

    handle
}
