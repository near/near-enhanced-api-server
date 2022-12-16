use crate::modules::nft;
use crate::{db_helpers, errors, types};

// TODO PHASE 2 pagination by artificial index added to assets__non_fungible_token_events
pub(crate) async fn get_nft_history(
    pool_explorer: &sqlx::Pool<sqlx::Postgres>,
    contract_id: &near_primitives::types::AccountId,
    token_id: &str,
    block: &db_helpers::Block,
    limit: u32,
) -> crate::Result<Vec<nft::schemas::HistoryItem>> {
    let query = r"
        SELECT
            event_kind::text cause,
            CASE WHEN execution_outcomes.status IN ('SUCCESS_VALUE', 'SUCCESS_RECEIPT_ID') THEN 'SUCCESS'
                ELSE 'FAILURE'
            END status,
            token_old_owner_account_id old_account_id,
            token_new_owner_account_id new_account_id,
            emitted_at_block_timestamp block_timestamp_nanos,
            block_height
        FROM assets__non_fungible_token_events
            JOIN blocks ON assets__non_fungible_token_events.emitted_at_block_timestamp = blocks.block_timestamp
            JOIN execution_outcomes ON assets__non_fungible_token_events.emitted_for_receipt_id = execution_outcomes.receipt_id
        WHERE token_id = $1
            AND emitted_by_contract_account_id = $2
            AND emitted_at_block_timestamp < $3::numeric(20, 0)
        ORDER BY emitted_at_block_timestamp DESC
        LIMIT $4::numeric(20, 0)
    ";
    let history_items = db_helpers::select_retry_or_panic::<super::models::NftHistoryInfo>(
        pool_explorer,
        query,
        &[
            token_id.to_string(),
            contract_id.to_string(),
            block.timestamp.to_string(),
            limit.to_string(),
        ],
    )
    .await?;

    let mut result: Vec<nft::schemas::HistoryItem> = vec![];
    for history in history_items {
        result.push(history.try_into()?);
    }
    Ok(result)
}

impl TryFrom<super::models::NftHistoryInfo> for nft::schemas::HistoryItem {
    type Error = errors::Error;

    fn try_from(info: super::models::NftHistoryInfo) -> crate::Result<Self> {
        Ok(Self {
            cause: info.cause,
            status: info.status,
            old_account_id: types::account_id::extract_account_id(&info.old_account_id)?
                .map(|account| account.into()),
            new_account_id: types::account_id::extract_account_id(&info.new_account_id)?
                .map(|account| account.into()),
            block_timestamp_nanos: types::numeric::to_u64(&info.block_timestamp_nanos)?.into(),
            block_height: types::numeric::to_u64(&info.block_height)?.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_nft_history() {
        let pool_explorer = init_explorer_db().await;
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "293708:1";

        let history = get_nft_history(&pool_explorer, &contract, token, &block, 10).await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_nft_history_with_failed_receipts() {
        let pool_explorer = init_explorer_db().await;
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("thebullishbulls.near").unwrap();
        let token = "1349";

        let history = get_nft_history(&pool_explorer, &contract, token, &block, 10).await;
        insta::assert_debug_snapshot!(history);
    }

    #[tokio::test]
    async fn test_nft_history_nft_does_not_exist() {
        let pool_explorer = init_explorer_db().await;
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("x.paras.near").unwrap();
        let token = "no_such_token";

        let history = get_nft_history(&pool_explorer, &contract, token, &block, 10)
            .await
            .unwrap();
        assert!(history.is_empty());
    }
}
