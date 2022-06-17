use std::time::Duration;

use near_enhanced_api::{config, start};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let url = &std::env::var("DATABASE_URL").expect("failed to get database url");
    let pool = sqlx::PgPool::connect(url)
        .await
        .expect("failed to connect to the database");
    start(config::Config::default(), pool);
    loop {
        tokio::time::sleep(Duration::from_secs(100)).await;
    }
}
