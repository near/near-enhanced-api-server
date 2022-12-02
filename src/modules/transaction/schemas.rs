use crate::types;
use paperclip::actix::Apiv2Schema;
use validator::Validate;

// *** Requests ***

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionRequest {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub transaction_hash: Option<types::TransactionHash>,
    pub receipt_id: Option<types::ReceiptId>,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionsRequest {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_id: Option<types::AccountId>,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ReceiptsRequest {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub tx_hash: Option<types::TransactionHash>,
}
#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ActionReceiptsRequest {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_id: Option<types::AccountId>,
}

// *** Responses ***
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct TransactionResponse {
    pub transaction: Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct TransactionsResponse {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct ReceiptsResponse {
    pub receipts: Vec<Receipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct ActionReceiptsResponse {
    pub action_receipts: Vec<ActionReceipt>,
}

// *** Types ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Transaction {
    pub signer_id: String,
    pub public_key: String,
    pub receiver_id: String,
    pub block_hash: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Receipt {
    /// An issuer account_id of a particular receipt.
    /// `predecessor_id` could be either `Transaction` `signer_id` or intermediate contract's `account_id`.
    pub predecessor_id: types::AccountId,
    /// `receiver_id` is a receipt destination.
    pub receiver_id: types::AccountId,
    /// An unique id for the receipt
    pub receipt_id: types::ReceiptId,
    /// A receipt type
    pub receipt: ReceiptEnum,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub enum ReceiptEnum {
    Action(ActionReceipt),
    // Todo Define DataReceipt
    // Data(DataReceipt),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct ActionReceipt {
    pub signer_id: types::AccountId,
    /// Todo change this to a crypto hash or Public Key type
    pub signer_public_key: types::AccountId,
    pub gas_price: types::U128,
    pub output_data_receivers: Vec<DataReceiver>,
    /// Todo Change type from string to crypto hash
    pub input_data_ids: Vec<String>,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct DataReceiver {
    /// todo change type from string to crypto hash
    pub data_id: String,
    pub receiver_id: types::AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub enum Action {
    /// Todo Add more actions
    CreateAccount(CreateAccountAction),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct CreateAccountAction {}
