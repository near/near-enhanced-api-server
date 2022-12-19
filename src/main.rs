use actix_cors::Cors;
use actix_web::{App, HttpServer, ResponseError};
use actix_web_prom::PrometheusMetricsBuilder;
use actix_web_validator::PathConfig;
use paperclip::actix::{web, OpenApiExt};
pub(crate) use sqlx::types::BigDecimal;

mod config;
mod db_helpers;
mod errors;
mod modules;
mod rpc_helpers;
mod types;

pub(crate) const LOGGER_MSG: &str = "near_enhanced_api";

pub(crate) type Result<T> = std::result::Result<T, errors::Error>;

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
            actix_web::http::header::CONTENT_TYPE,
        ])
        .allowed_header("x-api-key")
        .max_age(3600)
}

fn get_api_base_path() -> String {
    std::env::var("API_BASE_PATH").unwrap_or_else(|_| "".to_string())
}

async fn playground_ui() -> impl actix_web::Responder {
    let base_path = get_api_base_path();
    actix_web::HttpResponse::Ok()
        .insert_header(actix_web::http::header::ContentType::html())
        .body(
            format!(r#"<!doctype html>
                <html lang="en">
                  <head>
                    <meta charset="utf-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">
                    <title>NEAR Enhanced API powered by Pagoda - Playground</title>
                    <!-- Embed elements Elements via Web Component -->
                    <script src="https://unpkg.com/@stoplight/elements/web-components.min.js"></script>
                    <link rel="stylesheet" href="https://unpkg.com/@stoplight/elements/styles.min.css">
                  </head>
                  <body>

                    <elements-api
                      apiDescriptionUrl="{base_path}/spec/v3.json"
                      router="hash"
                      layout="sidebar"
                    />

                  </body>
                </html>"#),
        )
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let env_filter = tracing_subscriber::EnvFilter::new(
        std::env::var("RUST_LOG")
            .as_deref()
            .unwrap_or("info,near=info,near_jsonrpc_client=warn,near_enhanced_api=debug"),
    );

    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();
    tracing::debug!(
        target: crate::LOGGER_MSG,
        "NEAR Enhanced API Server is initializing..."
    );

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let db_url = &std::env::var("DATABASE_URL")
        .expect("failed to get database url from DATABASE_URL env variable");

    // See https://docs.rs/sqlx/latest/sqlx/struct.Pool.html#2-connection-limits-mysql-mssql-postgres
    // for setting connection limits.
    let db_max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "97".to_string())
        .parse()
        .expect("Failed to parse DATABASE_MAX_CONNECTIONS value as u32");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_max_connections)
        .connect(db_url)
        .await
        .expect("failed to connect to the database");

    let url_balances = &std::env::var("DATABASE_URL_BALANCES")
        .expect("failed to get database url from DATABASE_URL_BALANCES env variable");
    let pool_balances = sqlx::postgres::PgPoolOptions::new()
        .max_connections(db_max_connections)
        .connect(url_balances)
        .await
        .expect("failed to connect to the balances database");

    let rpc_url =
        &std::env::var("RPC_URL").expect("failed to get RPC url from RPC_URL env variable");
    let rpc_client = near_jsonrpc_client::JsonRpcClient::connect(rpc_url);

    let config::Config {
        addr,
        cors_allowed_origins,
        limits,
    } = config::Config::default();

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

        let path_config = PathConfig::default().error_handler(|err, _| {
            let error_message = err.to_string();
            actix_web::error::InternalError::from_response(
                err,
                errors::Error::from_error_kind(errors::ErrorKind::InvalidInput(error_message))
                    .error_response(),
            )
            .into()
        });

        let mut spec = paperclip::v2::models::DefaultApiRaw::default();
        if let Ok(api_server_public_host) = std::env::var("API_SERVER_PUBLIC_HOST") {
            spec.schemes
                .insert(paperclip::v2::models::OperationProtocol::Https);
            spec.host = Some(api_server_public_host);
        }
        let base_path = get_api_base_path();
        spec.base_path = Some(base_path.clone());
        spec.info = paperclip::v2::models::Info {
            version: "0.1".into(),
            title: "NEAR Enhanced API powered by Pagoda".into(),
            description: Some(format!(r#"Try out our newly released Enhanced APIs - Balances (in Beta) and get what you need for all kinds of balances and token information at ease.
Call Enhanced APIs using the endpoint in the API URL box, varies by Network.

https://near-testnet.api.pagoda.co{base_path}

https://near-mainnet.api.pagoda.co{base_path}

Grab your API keys and give it a try! We will be adding more advanced Enhanced APIs in our offering, so stay tuned. Get the data you need without extra processing, NEAR Blockchain data query has never been easier!

We would love to hear from you on the data APIs you need, please leave feedback using the widget in the lower-right corner."#)),
            ..Default::default()
        };

        let mut app = App::new()
            .app_data(json_config)
            .app_data(path_config)
            .wrap(actix_web::middleware::Logger::default())
            .wrap(prometheus.clone())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(db_helpers::DBWrapper {
                pool: pool_balances.clone(),
            }))
            .app_data(web::Data::new(rpc_client.clone()))
            .wrap(get_cors(&cors_allowed_origins))
            .route("/", actix_web::web::get().to(playground_ui))
            .wrap_api_with_spec(spec);

        app = app.configure(modules::native::register_services);
        app = app.configure(modules::ft::register_services);
        app = app.configure(modules::nft::register_services);
        app = app.configure(modules::transaction::register_services);
        app.with_json_spec_at(format!("{base_path}/spec/v2.json").as_str())
            .with_json_spec_v3_at(format!("{base_path}/spec/v3.json").as_str())
            .build()
    })
    .bind(addr)
    .unwrap()
    .shutdown_timeout(5)
    .run();

    tracing::debug!(
        target: crate::LOGGER_MSG,
        "NEAR Enhanced API Server is starting..."
    );

    server.await
}
