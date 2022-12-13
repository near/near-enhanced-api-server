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
    pub transaction_hash: types::TransactionHash,
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
    pub action_receipts: Vec<Receipt>,
}

// *** Types ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Transaction {
    /// Transaction Hash of the the transction
    pub transaction_hash: String,
    /// An account on which behalf transaction is signed
    pub signer_account_id: String,
    /// An access key which was used to sign the original transaction
    pub signer_public_key: String,
    /// Receiver account for this transaction
    pub receiver_account_id: String,
    /// The hash of the block this transaction was included in
    pub block_hash: String,
    /// A list of actions to be applied
    pub actions: Vec<Action>,
    /// Timestamp when the transaction was finalized
    pub timestamp: u128,
    /// Transaction cost in Yocto Near
    pub total_gas_cost: u128,
    /// Amount of Near transferred during this transaction
    pub amount: u128,
    /// Status of the Transaction. Finalized | Pending | Failed
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Action {
    /// Transaction Hash
    pub transaction_hash: String,
    /// The index of the action in the transaction
    pub index_in_transaction: String,
    /// Type of action
    pub action_kind: ActionType,
    /// Arguments for the action
    pub args: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Receipt {
    /// An unique id for the receipt
    pub receipt_id: String,
    /// Transaction that created this receipt
    pub originated_from_transaction_hash: Option<String>,
    /// An issuer account_id of a particular receipt.
    /// `predecessor_id` could be either `Transaction` `signer_id` or intermediate contract's `account_id`.
    pub predecessor_account_id: String,
    /// `receiver_id` is a receipt destination.
    pub receiver_account_id: String,
    /// List of actions
    pub actions: Vec<ActionReceipt>,
    /// A receipt kind
    pub receipt_kind: String,
    /// Status of the receipt. Success | Failure | Unknown
    pub status: String,
    /// Block timestamp when the receipt was finalized
    pub block_timestamp: Option<u128>,
    /// total gas burnt applying this receipt
    pub gas_burnt: Option<types::U128>,
    /// Near tokens burnt while applying this receipt
    pub tokens_burnt: Option<types::U128>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub enum ReceiptEnum {
    Action(ActionReceipt),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct ActionReceipt {
    /// A signer of the original transaction
    pub signer_account_id: String,
    /// An access key which was used to sign the original transaction
    pub signer_public_key: String,
    /// A gas_price which has been used to buy gas in the original transaction
    pub gas_price: types::U128,
    /// A list of actions to process when all input_data_ids are filled
    pub actions: Vec<ActionType>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub enum ActionType {
    /// Todo Add more actions
    CreateAccount(CreateAccountAction),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]

pub struct CreateAccountAction {}
