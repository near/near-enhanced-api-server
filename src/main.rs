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
        ])
        .allowed_header(actix_web::http::header::CONTENT_TYPE)
        .max_age(3600)
}

async fn playground_ui() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok()
        .insert_header(actix_web::http::header::ContentType::html())
        .body(
            r#"<!doctype html>
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
                      apiDescriptionUrl="/api/spec/v3.json"
                      router="hash"
                      layout="sidebar"
                    />

                  </body>
                </html>"#,
        )
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let env_filter = tracing_subscriber::EnvFilter::new(
        "info,near=info,near_jsonrpc_client=warn,near_enhanced_api=debug",
    );

    if let Ok(rust_log) = std::env::var("RUST_LOG") {
        if !rust_log.is_empty() {
            for directive in rust_log.split(',').filter_map(|s| match s.parse() {
                Ok(directive) => Some(directive),
                Err(err) => {
                    eprintln!("Ignoring directive `{}`: {}", s, err);
                    None
                }
            }) {
                env_filter = env_filter.add_directive(directive);
            }
        }
    }

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
    let pool = sqlx::PgPool::connect(db_url)
        .await
        .expect("failed to connect to the database");

    let url_balances = &std::env::var("DATABASE_URL_BALANCES")
        .expect("failed to get database url from DATABASE_URL_BALANCES env variable");
    let pool_balances = sqlx::PgPool::connect(url_balances)
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
    let api_server_public_host =
        std::env::var("API_SERVER_PUBLIC_HOST").unwrap_or_else(|_| addr.clone());

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
        spec.schemes
            .insert(paperclip::v2::models::OperationProtocol::Https);
        spec.schemes
            .insert(paperclip::v2::models::OperationProtocol::Http);
        spec.host = Some(api_server_public_host.clone());
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
            title: "NEAR Enhanced API powered by Pagoda".into(),
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

        app = app.configure(modules::coin::register_services);
        app = app.configure(modules::nft::register_services);

        app.with_json_spec_at("/api/spec/v2.json")
            .with_json_spec_v3_at("/api/spec/v3.json")
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
