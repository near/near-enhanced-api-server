use crate::BigDecimal;

/// *Transaction*
#[derive(sqlx::FromRow)]
pub(crate) struct Transaction {
    transaction_hash: String,
    included_in_block_hash: String,
    block_timestamp: BigDecimal,
    signer_account_id: String,
    signer_public_key: String,
    receiver_account_id: String,
    signature: String,
    status: String,
    converted_into_receipt_id: String,
    receipt_conversion_gas_burnt: BigDecimal,
    receipt_conversion_tokens_burnt: BigDecimal,
}

#[derive(sqlx::FromRow)]
pub(crate) struct TransactionAction {
    transaction_hash: String,
    index_in_transaction: String,
    action_kind: String,
    args: String,
}

// * Receipt*
#[derive(sqlx::FromRow)]
pub(crate) struct Receipt {
    receipt_id: String,
    included_in_block_hash: String,
    block_timestamp: BigDecimal,
    predecessor_account_id: String,
    receiver_account_id: String,
    receipt_kind: String,
    originated_from_transaction_hash: String
}

#[derive(sqlx::FromRow)]
pub(crate) struct ExecutionOutcomeReceipt {
    receipt_id: String,
    included_in_block_hash: String,
    block_timestamp: BigDecimal,
    gas_burnt: BigDecimal,
    tokens_burnt: BigDecimal,
    executer_account_id: String,
    status: String,
    shard_id: String
}

#[derive(sqlx::FromRow)]
pub(crate) struct ActionReceiptActions {
    receipt_id: String,
    index_in_action_receipt: BigDecimal,
    action_kind: String,
    args: String,
    predecessor_account_id: String,
    receiver_account_id: String,
    block_timestamp: BigDecimal
}