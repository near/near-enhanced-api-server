use actix_cors::Cors;
use actix_web::{App, HttpServer, ResponseError};
use num_traits::cast::ToPrimitive;
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    OpenApiExt,
};
use sqlx::types::BigDecimal;
use std::ops::Add;

mod api_models;
pub mod config;
mod db_models;
mod errors;
mod types;
mod utils;

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);
const RETRY_COUNT: usize = 10;

async fn native_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    request: web::Path<api_models::AccountBalanceRequest>,
    params: web::Query<api_models::QueryParams>,
) -> Result<Json<api_models::AccountBalanceResponse>, api_models::Error> {
    check_params(&params)?;
    let timestamp = get_timestamp_from_params(&pool, &params).await?;
    if !account_exists(&pool, &request.account_id.0, timestamp).await? {
        return Err(errors::ErrorKind::InvalidInput("account does not exist".to_string()).into());
    }

    let db_result =
            utils::select_retry_or_panic::<db_models::AccountChangesBalance>(
                &pool,
                r"WITH t AS (
                    SELECT affected_account_nonstaked_balance nonstaked, affected_account_staked_balance staked, changed_in_block_timestamp block_timestamp
                    FROM account_changes
                    WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
                    ORDER BY changed_in_block_timestamp DESC
                  )
                  SELECT * FROM t LIMIT 1
                 ",
                &[request.account_id.0.to_string(), timestamp.to_string()],
                RETRY_COUNT,
            ).await?;

    match db_result.first() {
        Some(row) => {
            //todo put into function
            let amount = (&row.nonstaked)
                .add(&row.staked)
                .to_string()
                .parse::<u128>()
                .expect("amount expected to be u128");
            let timestamp = row
                .block_timestamp
                .to_string()
                .parse::<u64>()
                .expect("timestamp expected to be u64");
            Ok(Json(api_models::AccountBalanceResponse {
                token_kind: "near".to_string(),
                token_id: "near".to_string(),
                amount: amount.into(),
                block_timestamp_nanos: timestamp.into(),
            }))
        }
        None => Err(errors::ErrorKind::InvalidInput("account does not exist".to_string()).into()),
    }
}

#[api_v2_operation]
/// Get the user's balance
///
/// This endpoint returns the balance of the given account_id,
/// for the specified token_contract_id | near.
async fn token_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    request: web::Path<api_models::AccountBalanceRequest>,
    params: web::Query<api_models::QueryParams>,
) -> Result<Json<api_models::AccountBalanceResponse>, api_models::Error> {
    if request.token_contract_id.to_string() == "near" {
        native_balance(pool, request, params).await
    } else {
        Err(errors::ErrorKind::NotImplemented(
            "FT and other stuff is not implemented yet".to_string(),
        )
        .into())
    }
}

fn check_params(params: &web::Query<api_models::QueryParams>) -> Result<(), api_models::Error> {
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

async fn account_exists(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> Result<bool, api_models::Error> {
    // for the given timestamp, account exists if
    // 1. we have at least 1 row at action_receipt_actions table
    // 2. last action_kind != DELETE_ACCOUNT
    Ok(utils::select_retry_or_panic::<db_models::ActionKind>(
        pool,
        r"SELECT action_kind::text FROM action_receipt_actions
          WHERE receipt_receiver_account_id = $1
              AND action_receipt_actions.receipt_included_in_block_timestamp <= $2::numeric(20, 0)
          ORDER BY receipt_included_in_block_timestamp DESC, index_in_action_receipt DESC
          LIMIT 1",
        &[account_id.to_string(), block_timestamp.to_string()],
        RETRY_COUNT,
    )
    .await?
    .first()
    .map(|kind| kind.action_kind != "DELETE_ACCOUNT")
    .unwrap_or_else(|| false))
}

async fn get_timestamp_from_params(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
    params: &web::Query<api_models::QueryParams>,
) -> Result<u64, api_models::Error> {
    if let Some(block_height) = params.block_height {
        utils::select_retry_or_panic::<db_models::BlockTimestamp>(
            pool,
            "SELECT block_timestamp FROM blocks WHERE block_height = $1::numeric(20, 0)",
            &[block_height.0.to_string()],
            RETRY_COUNT,
        )
        .await?
        .first()
        .and_then(|timestamp| timestamp.block_timestamp.to_u64())
        .ok_or_else(|| {
            errors::ErrorKind::DBError(format!("block_height {} is not found", block_height.0))
                .into()
        })
    } else if let Some(block_timestamp) = params.block_timestamp_nanos {
        let result_timestamp = utils::select_retry_or_panic::<db_models::BlockTimestamp>(
            pool,
            r"SELECT block_timestamp
              FROM blocks
              WHERE block_timestamp <= $1::numeric(20, 0)
              ORDER BY block_timestamp DESC
              LIMIT 1",
            &[block_timestamp.0.to_string()],
            RETRY_COUNT,
        )
        .await?
        .first()
        .and_then(|timestamp| timestamp.block_timestamp.to_u64());
        match result_timestamp {
            None => get_first_timestamp(pool).await,
            Some(x) => Ok(x),
        }
    } else {
        get_last_timestamp(pool).await
    }
}

async fn get_first_timestamp(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
) -> Result<u64, api_models::Error> {
    utils::select_retry_or_panic::<db_models::BlockTimestamp>(
        pool,
        r"SELECT block_timestamp
          FROM blocks
          ORDER BY block_timestamp
          LIMIT 1",
        &[],
        RETRY_COUNT,
    )
    .await?
    .first()
    .and_then(|timestamp| timestamp.block_timestamp.to_u64())
    .ok_or_else(|| errors::ErrorKind::DBError("blocks table is empty".to_string()).into())
}

async fn get_last_timestamp(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
) -> Result<u64, api_models::Error> {
    utils::select_retry_or_panic::<db_models::BlockTimestamp>(
        pool,
        r"SELECT block_timestamp
          FROM blocks
          ORDER BY block_timestamp DESC
          LIMIT 1",
        &[],
        RETRY_COUNT,
    )
        .await?
        .first()
        .and_then(|timestamp| timestamp.block_timestamp.to_u64())
        .ok_or_else(|| errors::ErrorKind::DBError("blocks table is empty".to_string()).into())
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
) -> actix_web::dev::ServerHandle {
    let config::Config {
        addr,
        cors_allowed_origins,
        limits,
    } = config;
    let server = HttpServer::new(move || {
        let json_config = web::JsonConfig::default()
            .limit(limits.input_payload_max_size)
            .error_handler(|err, _req| {
                let error_message = err.to_string();
                actix_web::error::InternalError::from_response(
                    err,
                    api_models::Error::from_error_kind(errors::ErrorKind::InvalidInput(
                        error_message,
                    ))
                    .error_response(),
                )
                .into()
            });

        App::new()
            .app_data(json_config)
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .wrap(get_cors(&cors_allowed_origins))
            .wrap_api()
            .service(
                web::resource("/account/{account_id}/coins/{token_contract_id}")
                    .route(web::get().to(token_balance)),
            )
            .with_json_spec_at("/api/spec")
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

// TODO we need to add absolute value column for this query
// go to rpc
// Token Balance for a given Address(FT)
// List of all the tokens with their balances
// /accounts/<account-id>/coins | pagination + timestamp
//

// SELECT 'nep141' as standard, emitted_by_contract_account_id as token_id, amount, emitted_by_contract_account_id as contract_account_id
// FROM assets__fungible_token_events
// WHERE

//
// [
//   {“standard”: “nearprotocol“, “token_id”: “NEAR“, “amount“: 10},
//   {“standard”: “nep141”, “token_id”: “USN“, “amount“: 10, “contract_account_id”: “<token-contract-id>“},
//   {“standard”: “nep245”, “token_id”: “MT_FROL_GOLD“, “amount“: 10, “contract_account_id”: “<token-contract-id>“},
//   {“standard”: “nep245”, “token_id”: “MT_FROL_SILVER“, “amount“: 1, “contract_account_id”: “<token-contract-id>“}
// ]
