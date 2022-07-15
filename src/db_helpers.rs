use sqlx::{Arguments, postgres::PgRow};

use crate::{BigDecimal, errors, types};


const DB_RETRY_COUNT: usize = 1;
const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

// temp solution to pass 2 different connection pools
pub(crate) struct DBWrapper {
    pub pool: sqlx::Pool<sqlx::Postgres>,
}

#[derive(sqlx::FromRow)]
struct BlockView {
    pub block_height: BigDecimal,
    pub block_timestamp: BigDecimal,
}

#[derive(sqlx::FromRow)]
struct ActionKindView {
    pub action_kind: String,
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

// TODO PHASE 2+ we are loosing +1 second here, it's painful. It could be computed much easier in Aurora DB
pub(crate) async fn does_account_exist(
    pool: &sqlx::Pool<sqlx::Postgres>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> crate::Result<bool> {
    // for the given timestamp, account exists if
    // 1. we have at least 1 row at action_receipt_actions table
    // 2. last successful action_kind != DELETE_ACCOUNT
    let query = r"
        SELECT action_kind::text
        FROM action_receipt_actions JOIN execution_outcomes ON action_receipt_actions.receipt_id = execution_outcomes.receipt_id
        WHERE receipt_predecessor_account_id = $1
            AND action_receipt_actions.receipt_included_in_block_timestamp <= $2::numeric(20, 0)
            AND execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID')
        ORDER BY receipt_included_in_block_timestamp DESC, index_in_action_receipt DESC
        LIMIT 1
     ";
    Ok(select_retry_or_panic::<ActionKindView>(
        pool,
        query,
        &[account_id.to_string(), block_timestamp.to_string()],
            )
        .await?
        .first()
        .map(|kind| kind.action_kind != "DELETE_ACCOUNT")
        .unwrap_or_else(|| false))
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

pub(crate) async fn get_last_block(
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> crate::Result<Block> {
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
