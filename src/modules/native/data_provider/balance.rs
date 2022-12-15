use crate::modules::native;
use crate::{db_helpers, errors, types};

// todo change to near_balance_events when we finish collecting the data
pub(crate) async fn get_near_balance(
    pool: &sqlx::Pool<sqlx::Postgres>,
    block: &db_helpers::Block,
    account_id: &near_primitives::types::AccountId,
) -> crate::Result<native::schemas::NearBalanceResponse> {
    let balances =
        db_helpers::select_retry_or_panic::<super::models::AccountChangesBalance>(
            pool,
            r"
                WITH t AS (
                    SELECT affected_account_nonstaked_balance + affected_account_staked_balance balance
                    FROM account_changes
                    WHERE affected_account_id = $1 AND changed_in_block_timestamp <= $2::numeric(20, 0)
                    ORDER BY changed_in_block_timestamp DESC
                )
                SELECT * FROM t LIMIT 1
            ",
            &[account_id.to_string(), block.timestamp.to_string()],
        ).await?;

    match balances.first() {
        Some(balance) => Ok(native::schemas::NearBalanceResponse {
            balance: native::schemas::NearBalance {
                amount: types::numeric::to_u128(&balance.balance)?.into(),
                metadata: super::metadata::get_near_metadata(),
            },
            block_timestamp_nanos: block.timestamp.into(),
            block_height: block.height.into(),
        }),
        None => Err(errors::ErrorKind::DBError(format!(
            "Could not find the data in account_changes table for account_id {}",
            account_id
        ))
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_near_balance() {
        let pool = init_db().await;
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("tomato.near").unwrap();
        let balance = get_near_balance(&pool, &block, &account).await;
        insta::assert_debug_snapshot!(balance);
    }
}
