use actix_cors::Cors;
use actix_web::{App, HttpServer, ResponseError};
use paperclip::actix::{
    api_v2_operation,
    web::{self, Json},
    OpenApiExt,
};
use sqlx::types::BigDecimal;
use sqlx::Row;

pub mod config;
mod errors;
mod models;
mod types;
mod utils;

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

#[api_v2_operation]
// Token Balance for a given Address(Native/NEAR)
// /accounts/<account-id>/tokens/NEAR | timestamp
// TODO do we want to serve staked balance?
// SELECT affected_account_nonstaked_balance
// FROM account_changes
// WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
// ORDER BY changed_in_block_timestamp DESC
// LIMIT 1;

// [{“token_kind”: “NEAR”, “token_id”: “NEAR“, “amount”: 100000}]
async fn token_balance(
    pool: web::Data<sqlx::Pool<sqlx::Postgres>>,
    body: Json<models::AccountBalanceRequest>,
) -> Result<Json<models::AccountBalanceResponse>, models::Error> {
    let query = r"SELECT affected_account_nonstaked_balance
                        FROM account_changes
                        WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
                        ORDER BY changed_in_block_timestamp DESC
                        "; // LIMIT 1 // todo it fails with timeout with the limit...
    let timestamp = body
        .block_timestamp
        .unwrap_or_else(utils::get_latest_timestamp_nanos);
    let a = utils::select_retry_or_panic(
        &pool,
        query,
        &[body.account_id.0.to_string(), timestamp.to_string()],
        10,
    )
    .await?;
    Ok(match a.first() {
        Some(x) => {
            let aaa: BigDecimal = x.get(0);
            let bbb = aaa.to_string().parse::<u128>().expect("sfe");
            Ok(Json(models::AccountBalanceResponse { amount: bbb }))
        }

        // todo sometimes it also does not exist (deleted), but we will show the balance
        None => Err(errors::ErrorKind::InvalidInput(
            "account does not exist".to_string(),
        )),
    }?)
}

fn get_cors(cors_allowed_origins: &[String]) -> Cors {
    let mut cors = Cors::permissive();
    if cors_allowed_origins != ["*".to_string()] {
        for origin in cors_allowed_origins {
            cors = cors.allowed_origin(origin);
        }
    }
    cors.allowed_methods(vec!["GET", "POST"])
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
                    models::Error::from_error_kind(errors::ErrorKind::InvalidInput(error_message))
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
            .service(web::resource("/account/balance").route(web::post().to(token_balance)))
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
