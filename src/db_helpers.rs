use sqlx::{postgres::PgRow, Arguments};

use crate::{errors, types, BigDecimal};

// The DB replicas apply the WALs each X seconds (X=30 or 300 in our case, depend on replica).
// If the SELECT query started right before WAL started to apply, the query is cancelled.
// That's why we need to try the second time.
// If it hits the limit again, it makes to sense to try run it the third time,
// 99% we will hit the limit again, that's why we have 2 here
const DB_RETRY_COUNT: usize = 2;

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

// temp solution to pass 2 different connection pools
pub struct DBWrapper {
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

#[derive(sqlx::FromRow)]
struct BlockView {
    pub block_height: BigDecimal,
    pub block_timestamp: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct AccountId {
    pub account_id: String,
}

pub(crate) struct Block {
    pub timestamp: u64,
    pub height: u64,
}

impl TryFrom<&BlockView> for Block {
    type Error = errors::Error;

    fn try_from(block: &BlockView) -> crate::Result<Self> {
        Ok(Self {
            timestamp: types::numeric::to_u64(&block.block_timestamp)?,
            height: types::numeric::to_u64(&block.block_height)?,
        })
    }
}

pub(crate) async fn get_block_from_params(
    pool: &sqlx::Pool<sqlx::Postgres>,
    params: &types::query_params::BlockParams,
) -> crate::Result<Block> {
    if let Some(block_height) = params.block_height {
        match select_retry_or_panic::<BlockView>(
            pool,
            "SELECT block_height, block_timestamp FROM blocks WHERE block_height = $1::numeric(20, 0)",
            &[block_height.0.to_string()],
                    )
            .await?
            .first() {
            None => Err(errors::ErrorKind::DBError(format!("block_height {} is not found", block_height.0)).into()),
            Some(block) => Ok(Block::try_from(block)?)
        }
    } else if let Some(block_timestamp) = params.block_timestamp_nanos {
        match select_retry_or_panic::<BlockView>(
            pool,
            r"SELECT block_height, block_timestamp
              FROM blocks
              WHERE block_timestamp <= $1::numeric(20, 0)
              ORDER BY block_timestamp DESC
              LIMIT 1",
            &[block_timestamp.0.to_string()],
        )
        .await?
        .first()
        {
            None => get_first_block(pool).await,
            Some(block) => Ok(Block::try_from(block)?),
        }
    } else {
        get_last_block(pool).await
    }
}

async fn get_first_block(pool: &sqlx::Pool<sqlx::Postgres>) -> crate::Result<Block> {
    match select_retry_or_panic::<BlockView>(
        pool,
        r"SELECT block_height, block_timestamp
          FROM blocks
          ORDER BY block_timestamp
          LIMIT 1",
        &[],
    )
    .await?
    .first()
    {
        None => Err(errors::ErrorKind::DBError("blocks table is empty".to_string()).into()),
        Some(block) => Ok(Block::try_from(block)?),
    }
}

pub(crate) async fn get_last_block(pool: &sqlx::Pool<sqlx::Postgres>) -> crate::Result<Block> {
    match select_retry_or_panic::<BlockView>(
        pool,
        r"SELECT block_height, block_timestamp
          FROM blocks
          ORDER BY block_timestamp DESC
          LIMIT 1",
        &[],
    )
    .await?
    .first()
    {
        None => Err(errors::ErrorKind::DBError("blocks table is empty".to_string()).into()),
        Some(block) => Ok(Block::try_from(block)?),
    }
}

pub(crate) async fn select_retry_or_panic<T: Send + Unpin + for<'r> sqlx::FromRow<'r, PgRow>>(
    pool: &sqlx::Pool<sqlx::Postgres>,
    query: &str,
    substitution_items: &[String],
) -> Result<Vec<T>, errors::ErrorKind> {
    let mut interval = INTERVAL;
    let mut retry_attempt = 0usize;

    tracing::info!(
        target: crate::LOGGER_MSG,
        "DB request:\n{}\nParams:{}",
        query,
        substitution_items.join(", ")
    );

    loop {
        if retry_attempt == DB_RETRY_COUNT {
            return Err(errors::ErrorKind::DBError(format!(
                "Failed to perform query to database after {} attempts. Stop trying.",
                DB_RETRY_COUNT
            )));
        }
        retry_attempt += 1;

        let mut args = sqlx::postgres::PgArguments::default();
        for item in substitution_items {
            args.add(item);
        }

        match sqlx::query_as_with::<_, T, _>(query, args)
            .fetch_all(pool)
            .await
        {
            Ok(res) => return Ok(res),
            Err(async_error) => {
                tracing::warn!(
                    target: crate::LOGGER_MSG,
                    "Error occurred during {:#?}:\nFailed SELECT:\n{}Params:{}\n Retrying in {} milliseconds...",
                    async_error,
                    query,
                    substitution_items.join(", "),
                    interval.as_millis(),
                );
                tokio::time::sleep(interval).await;
                if interval < MAX_DELAY_TIME {
                    interval *= 2;
                }
            }
        }
    }
}
