use crate::types;
use paperclip::actix::Apiv2Schema;
use validator::Validate;

// *** Requests ***

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionRequest {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub transaction_hash: types::TransactionHash,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionRequestByReceiptId {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub receipt_id: types::ReceiptId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionRequestByAccountId {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionsRequestByAccountId {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionsRequestByAccountIdAndContractId {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_id: types::AccountId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ReceiptsRequestsByTransactionHash {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub receipt_id: types::ReceiptId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ActionReceiptsRequestByAccountId {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ActionReceiptsRequestAccountIdAndContractId {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_id: types::AccountId,
}

// *** Responses ***
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct TransactionResponse {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct TransactionsResponse {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct ReceiptsResponse {}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct ActionReceiptsResponse {}
