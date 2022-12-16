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

#[derive(Debug)]
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

pub(crate) fn timestamp_to_event_index(timestamp: u64) -> u128 {
    timestamp as u128 * 100_000_000 * 100_000_000
}

pub(crate) fn event_index_to_timestamp(event_index: u128) -> u64 {
    (event_index / (100_000_000 * 100_000_000)) as u64
}

pub(crate) async fn get_block_from_pagination(
    pool_explorer: &sqlx::Pool<sqlx::Postgres>,
    pagination: &types::query_params::Pagination,
) -> crate::Result<Block> {
    if let Some(event_index) = pagination.after_event_index {
        checked_get_block(
            pool_explorer,
            &types::query_params::BlockParams {
                block_timestamp_nanos: Some(event_index_to_timestamp(event_index).into()),
                block_height: None,
            },
        )
        .await
    } else {
        checked_get_block(
            pool_explorer,
            &types::query_params::BlockParams {
                block_timestamp_nanos: None,
                block_height: None,
            },
        )
        .await
    }
}

/// Validates block_params received from the user, sets the default value if none was provided
pub(crate) async fn checked_get_block(
    pool_explorer: &sqlx::Pool<sqlx::Postgres>,
    block_params: &types::query_params::BlockParams,
) -> crate::Result<Block> {
    if block_params.block_height.is_some() && block_params.block_timestamp_nanos.is_some() {
        return Err(errors::ErrorKind::InvalidInput(
            "Both block_height and block_timestamp_nanos found. Please provide only one of values"
                .to_string(),
        )
        .into());
    }

    if let Some(block_height) = block_params.block_height {
        match select_retry_or_panic::<BlockView>(
            pool_explorer,
            r"SELECT block_height, block_timestamp
              FROM blocks
              WHERE block_height = $1::numeric(20, 0)",
            &[block_height.0.to_string()],
        )
        .await?
        .first()
        {
            None => Err(errors::ErrorKind::InvalidInput(format!(
                "block_height {} is not found",
                block_height.0
            ))
            .into()),
            Some(block) => Ok(Block::try_from(block)?),
        }
    } else if let Some(block_timestamp) = block_params.block_timestamp_nanos {
        match select_retry_or_panic::<BlockView>(
            pool_explorer,
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
            Some(block) => Ok(Block::try_from(block)?),
            None => get_first_block(pool_explorer).await,
        }
    } else {
        get_last_block(pool_explorer).await
    }
}

async fn get_first_block(pool_explorer: &sqlx::Pool<sqlx::Postgres>) -> crate::Result<Block> {
    match select_retry_or_panic::<BlockView>(
        pool_explorer,
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

pub(crate) async fn get_last_block(
    pool_explorer: &sqlx::Pool<sqlx::Postgres>,
) -> crate::Result<Block> {
    match select_retry_or_panic::<BlockView>(
        pool_explorer,
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

pub(crate) async fn get_previous_block(
    pool_explorer: &sqlx::Pool<sqlx::Postgres>,
    current_block_timestamp: u64,
) -> crate::Result<Block> {
    match select_retry_or_panic::<BlockView>(
        pool_explorer,
        r"SELECT block_height, block_timestamp
           FROM blocks
           WHERE block_timestamp < $1::numeric(20, 0)
           ORDER BY block_timestamp DESC
           LIMIT 1",
        &[current_block_timestamp.to_string()],
    )
    .await?
    .first()
    {
        None => Err(errors::ErrorKind::DBError("blocks table is empty".to_string()).into()),
        Some(block) => Ok(Block::try_from(block)?),
    }
}

pub(crate) async fn select_retry_or_panic<T: Send + Unpin + for<'r> sqlx::FromRow<'r, PgRow>>(
    pool_explorer: &sqlx::Pool<sqlx::Postgres>,
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
            .fetch_all(pool_explorer)
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
