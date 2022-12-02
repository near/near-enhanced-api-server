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
    pub transaction_hash: Option<types::TransactionHash>,
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
    /// An account on which behalf transaction is signed
    pub signer_id: String,
    /// A public key of the access key which was used to sign an account.
    /// Access key holds permissions for calling certain kinds of actions.
    pub public_key: String,
    /// Receiver account for this transaction
    pub receiver_id: String,
    /// The hash of the block in the blockchain on top of which the given transaction is valid
    pub block_hash: String,
    /// A list of actions to be applied
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
    /// A receipt type(Action Or Data Receipt)
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
    /// A signer of the original transaction
    pub signer_id: types::AccountId,
    /// Todo change this to a crypto hash or Public Key type
    /// An access key which was used to sign the original transaction
    pub signer_public_key: types::AccountId,
    /// A gas_price which has been used to buy gas in the original transaction
    pub gas_price: types::U128,
    /// If present, where to route the output data
    pub output_data_receivers: Vec<DataReceiver>,
    /// Todo Change type from string to crypto hash
    /// A list of the input data dependencies for this Receipt to process.
    /// If all `input_data_ids` for this receipt are delivered to the account
    /// that means we have all the `ReceivedData` input which will be than converted to a
    /// `PromiseResult::Successful(value)` or `PromiseResult::Failed`
    /// depending on `ReceivedData` is `Some(_)` or `None`
    pub input_data_ids: Vec<String>,
    /// A list of actions to process when all input_data_ids are filled
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
