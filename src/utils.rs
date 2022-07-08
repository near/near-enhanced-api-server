use std::str::FromStr;

use num_traits::ToPrimitive;
use sqlx::postgres::PgRow;
use sqlx::Arguments;
use tracing::{info, warn};

use crate::{api, api_models, errors, types, BigDecimal};

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);
pub(crate) const LOGGER_MSG: &str = "near_enhanced_api";

pub(crate) async fn select_retry_or_panic<T: Send + Unpin + for<'r> sqlx::FromRow<'r, PgRow>>(
    pool: &sqlx::Pool<sqlx::Postgres>,
    query: &str,
    substitution_items: &[String],
    retry_count: usize,
) -> Result<Vec<T>, errors::ErrorKind> {
    let mut interval = INTERVAL;
    let mut retry_attempt = 0usize;

    info!(
        target: LOGGER_MSG,
        "DB request:\n{}\nParams:{}",
        query,
        substitution_items.join(", ")
    );

    loop {
        if retry_attempt == retry_count {
            return Err(errors::ErrorKind::DBError(format!(
                "Failed to perform query to database after {} attempts. Stop trying.",
                retry_count
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
                warn!(
                    target: LOGGER_MSG,
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

pub(crate) fn to_u128(x: &BigDecimal) -> api_models::Result<u128> {
    x.to_string().parse().map_err(|e| {
        errors::ErrorKind::InternalError(format!("Failed to parse u128 {}: {}", x, e)).into()
    })
}

pub(crate) fn to_i128(x: &BigDecimal) -> api_models::Result<i128> {
    x.to_string().parse().map_err(|e| {
        errors::ErrorKind::InternalError(format!("Failed to parse i128 {}: {}", x, e)).into()
    })
}

pub(crate) fn to_u64(x: &BigDecimal) -> api_models::Result<u64> {
    x.to_u64().ok_or_else(|| {
        errors::ErrorKind::InternalError(format!("Failed to parse u64 {}", x)).into()
    })
}

pub(crate) fn base64_to_string(
    value: &Option<types::Base64VecU8>,
) -> api_models::Result<Option<String>> {
    Ok(if let Some(v) = value {
        Some(serde_json::to_string(&v)?)
    } else {
        None
    })
}

pub(crate) fn extract_account_id(
    account_id: &str,
) -> api_models::Result<Option<near_primitives::types::AccountId>> {
    if account_id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(near_primitives::types::AccountId::from_str(
            account_id,
        )?))
    }
}

pub(crate) fn check_block_params(params: &api_models::BlockParams) -> api_models::Result<()> {
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

pub(crate) fn check_limit(limit_param: Option<u32>) -> api_models::Result<()> {
    if let Some(limit) = limit_param {
        if limit > crate::MAX_PAGE_LIMIT || limit == 0 {
            return Err(errors::ErrorKind::InvalidInput(format!(
                "Limit should be in range [1, {}]",
                crate::MAX_PAGE_LIMIT
            ))
            .into());
        }
    }
    Ok(())
}

pub(crate) async fn check_account_exists(
    pool: &sqlx::Pool<sqlx::Postgres>,
    account_id: &near_primitives::types::AccountId,
    block_timestamp: u64,
) -> api_models::Result<()> {
    if !api::does_account_exist(pool, account_id, block_timestamp).await? {
        Err(errors::ErrorKind::InvalidInput(format!(
            "account_id {} does not exist at block_timestamp {}",
            account_id, block_timestamp
        ))
        .into())
    } else {
        Ok(())
    }
}

pub(crate) async fn check_and_get_history_pagination_params(
    pool: &sqlx::Pool<sqlx::Postgres>,
    pagination_params: &api_models::HistoryPaginationParams,
) -> api_models::Result<types::HistoryPagination> {
    check_limit(pagination_params.limit)?;
    // if pagination_params.after_block_height.is_some() && pagination_params.after_timestamp_nanos.is_some() {
    //     return Err(errors::ErrorKind::InvalidInput(
    //         "Both block_height and block_timestamp_nanos found. Please provide only one of values"
    //             .to_string(),
    //     )
    //         .into());
    // }
    // TODO PHASE 2 take the block from pagination_params
    let block = api::get_last_block(pool).await?;
    Ok(types::HistoryPagination {
        block_height: block.height,
        block_timestamp: block.timestamp,
        limit: pagination_params.limit.unwrap_or(crate::DEFAULT_PAGE_LIMIT),
    })
}
