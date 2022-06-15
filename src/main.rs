use near_enhanced_api::start;
use std::time::Duration;

use near_enhanced_api::config;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let url = &std::env::var("DATABASE_URL").expect("failed to get database url");
    let pool = sqlx::PgPool::connect(url).await.expect("asd");
    start(config::Config::default(), pool);
    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}
