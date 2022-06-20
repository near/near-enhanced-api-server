use actix_cors::Cors;
use actix_web::{App, HttpServer, ResponseError};
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    OpenApiExt,
};
use sqlx::types::BigDecimal;
use std::str::FromStr;

mod api_models;
pub mod config;
mod db_models;
mod errors;
mod rpc_calls;
mod types;
mod utils;

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);
const RETRY_COUNT: usize = 10;

// todo write creds to the doc

#[api_v2_operation]
/// Get the user's balance
///
/// This endpoint returns the balance of the given account_id,
/// for the specified token_contract_id | near.
async fn native_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    request: web::Path<api_models::AccountNearBalanceRequestForContract>,
    params: web::Query<api_models::QueryParams>,
) -> api_models::Result<Json<api_models::AccountBalanceResponseForContract>> {
    check_params(&params)?;
    let block = get_block_from_params(&pool, &params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let db_result =
            utils::select_retry_or_panic::<db_models::AccountChangesBalance>(
                &pool,
                r"WITH t AS (
                    SELECT affected_account_nonstaked_balance nonstaked, affected_account_staked_balance staked
                    FROM account_changes
                    WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
                    ORDER BY changed_in_block_timestamp DESC
                  )
                  SELECT * FROM t LIMIT 1
                 ",
                &[request.account_id.0.to_string(), block.timestamp.to_string()],
                RETRY_COUNT,
            ).await?;

    match db_result.first() {
        Some(balance) => {
            // TODO support nonstaked, staked amounts
            let amount = utils::to_u128(&balance.nonstaked)? + utils::to_u128(&balance.staked)?;
            Ok(Json(api_models::AccountBalanceResponseForContract {
                balances: vec![api_models::CoinInfo {
                    standard: "nearprotocol".to_string(),
                    token_id: "near".to_string(),
                    amount: amount.into(),
                }],
                block_timestamp_nanos: types::U64::from(block.timestamp),
                block_height: types::U64::from(block.height),
            }))
        }
        None => Err(errors::ErrorKind::DBError(format!(
            "Could not find the data in account_changes table for account_id {}",
            request.account_id.0
        ))
        .into()),
    }
}

#[api_v2_operation]
/// Get the user's balance
///
/// This endpoint returns the balance of the given account_id,
/// for the specified token_contract_id | near.
// [
//   {“standard”: “nep141”, “token_id”: “USN“, “amount“: 10},
//   {“standard”: “nepXXX”, “token_id”: “MT_FROL_GOLD“, “amount“: 10},
//   {“standard”: “nepXXX”, “token_id”: “MT_FROL_SILVER“, “amount“: 1},
// ]
async fn ft_balance_by_contract(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    rpc_client: web::Data<near_jsonrpc_client::JsonRpcClient>,
    request: web::Path<api_models::AccountBalanceRequestForContract>,
    params: web::Query<api_models::QueryParams>,
) -> api_models::Result<Json<api_models::AccountBalanceResponseForContract>> {
    if request.token_contract_id.to_string() == "near" {
        return Err(errors::ErrorKind::InvalidInput(
            "For native balance, please use NEAR (uppercase)".to_string(),
        )
        .into());
    }
    check_params(&params)?;
    let block = get_block_from_params(&pool, &params).await?;
    check_account_exists(&pool, &request.account_id.0, block.timestamp).await?;

    let db_result = utils::select_retry_or_panic::<db_models::AccountId>(
        &pool,
        r"SELECT DISTINCT emitted_by_contract_account_id
              FROM assets__fungible_token_events
              WHERE token_old_owner_account_id = $1 OR token_new_owner_account_id = $1
             ",
        &[
            request.account_id.0.to_string(),
            block.timestamp.to_string(),
        ],
        RETRY_COUNT,
    )
    .await?
    .iter()
    .filter_map(|contract| {
        match near_primitives::types::AccountId::from_str(&contract.account_id) {
            Ok(contract_id) => {
                let a = rpc_calls::get_balance(
                    &rpc_client,
                    contract_id,
                    request.account_id.0.clone(),
                    block.height,
                );
                Some(123)
            }
            Err(_) => None,
        }
    });

    todo!("not implemented yet");
}

fn check_params(params: &web::Query<api_models::QueryParams>) -> api_models::Result<()> {
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

async fn check_account_exists(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> api_models::Result<()> {
    if !account_exists(pool, account_id, block_timestamp).await? {
        Err(errors::ErrorKind::InvalidInput(format!(
            "account_id {} does not exist at block_timestamp {}",
            account_id, block_timestamp
        ))
        .into())
    } else {
        Ok(())
    }
}

async fn account_exists(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> api_models::Result<bool> {
    // for the given timestamp, account exists if
    // 1. we have at least 1 row at action_receipt_actions table
    // 2. last successful action_kind != DELETE_ACCOUNT
    // TODO we are loosing +1 second here, it's painful
    Ok(utils::select_retry_or_panic::<db_models::ActionKind>(
        pool,
        r"SELECT action_kind::text
          FROM action_receipt_actions JOIN execution_outcomes ON action_receipt_actions.receipt_id = execution_outcomes.receipt_id
          WHERE receipt_predecessor_account_id = $1
              AND action_receipt_actions.receipt_included_in_block_timestamp <= $2::numeric(20, 0)
              AND execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID')
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

async fn get_block_from_params(
    pool: &web::Data<sqlx::Pool<sqlx::Postgres>>,
    params: &web::Query<api_models::QueryParams>,
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

        App::new()
            .app_data(json_config)
            .wrap(actix_web::middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(rpc_client.clone()))
            .wrap(get_cors(&cors_allowed_origins))
            .wrap_api()
            .service(
                web::resource("/accounts/{account_id}/coins/NEAR")
                    .route(web::get().to(native_balance)),
            )
            // .service(
            //     web::resource("/accounts/{account_id}/coins/near")
            //         .route(web::get().to(native_balance)),
            // )
            // todo NEAR will go here and fail    near
            .service(
                web::resource("/accounts/{account_id}/coins/{token_contract_id}")
                    .route(web::get().to(ft_balance_by_contract)),
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
