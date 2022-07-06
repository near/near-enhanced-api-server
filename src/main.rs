use std::time::Duration;

use near_enhanced_api::{config, start};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let url = &std::env::var("DATABASE_URL").expect("failed to get database url");
    let pool = sqlx::PgPool::connect(url)
        .await
        .expect("failed to connect to the database");

    let url_balances = &std::env::var("DATABASE_URL_BALANCES").expect("failed to get database url");
    let pool_balances = sqlx::PgPool::connect(url_balances)
        .await
        .expect("failed to connect to the balances database");
    let rpc_client =
        near_jsonrpc_client::JsonRpcClient::connect("https://archival-rpc.mainnet.near.org");
    start(config::Config::default(), pool, pool_balances, rpc_client);
    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}

// todo add overflow docs everywhere
// todo page + limit. By timestamp/height
// todo think about pagination/sorting, create the doc with available options
