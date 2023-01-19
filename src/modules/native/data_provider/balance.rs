use crate::modules::native;
use crate::{db_helpers, errors, types};

pub(crate) async fn get_near_balance(
    db_helpers::BalancesPool(pool_balances): &db_helpers::BalancesPool,
    block: &db_helpers::Block,
    account_id: &near_primitives::types::AccountId,
) -> crate::Result<native::schemas::NearBalanceResponse> {
    let balances = db_helpers::select_retry_or_panic::<super::models::Balance>(
        pool_balances,
        r"
                WITH t AS (
                    SELECT absolute_nonstaked_amount + absolute_staked_amount balance
                    FROM near_balance_events
                    WHERE affected_account_id = $1 AND block_timestamp <= $2::numeric(20, 0)
                    ORDER BY block_timestamp DESC
                )
                SELECT * FROM t LIMIT 1
            ",
        &[account_id.to_string(), block.timestamp.to_string()],
    )
    .await?;

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
            "Could not find the data in near_balance_events table for account_id {}",
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
        let pool_balances = init_balances_db().await;
        let block = get_block();
        let account = near_primitives::types::AccountId::from_str("tomato.near").unwrap();
        let balance = get_near_balance(&pool_balances, &block, &account).await;
        insta::assert_debug_snapshot!(balance);
    }
}
