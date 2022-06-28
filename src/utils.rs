use num_traits::ToPrimitive;
use sqlx::postgres::PgRow;
use sqlx::Arguments;

use crate::{api_models, errors, types, BigDecimal};

const INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);
const MAX_DELAY_TIME: std::time::Duration = std::time::Duration::from_secs(120);

// TODO we actually don't need retries, right?
pub(crate) async fn select_retry_or_panic<T: Send + Unpin + for<'r> sqlx::FromRow<'r, PgRow>>(
    pool: &sqlx::Pool<sqlx::Postgres>,
    query: &str,
    substitution_items: &[String],
    retry_count: usize,
) -> Result<Vec<T>, errors::ErrorKind> {
    let mut interval = INTERVAL;
    let mut retry_attempt = 0usize;

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
            println!("{}", item);
            args.add(item);
        }

        match sqlx::query_as_with::<_, T, _>(query, args)
            .fetch_all(pool)
            .await
        {
            Ok(res) => return Ok(res),
            Err(async_error) => {
                // todo we print here select with non-filled placeholders. It would be better to get the final select statement here
                println!(
                         "Error occurred during {}:\nFailed SELECT:\n{}\n Retrying in {} milliseconds...",
                         async_error,
                    query,
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

// pub(crate) fn to_i128(x: &BigDecimal) -> Result<i128, errors::ErrorKind> {
//     x.to_string()
//         .parse()
//         .map_err(|e| errors::ErrorKind::InternalError(format!("Failed to parse i128 {}: {}", x, e)))
// }

pub(crate) fn string_to_i128(x: &String) -> api_models::Result<i128> {
    x.parse().map_err(|e| {
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
