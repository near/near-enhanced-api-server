use crate::types;
use paperclip::actix::Apiv2Schema;
use validator::Validate;

use near_primitives::{
    serialize::{base64_format, u128_dec_format},
    views,
};
// *** Requests ***

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionByTxHash {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub transaction_hash: String,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionByReceiptId {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub receipt_id: String,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionsByAccountId {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
}
#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct TransactionsByAccountIdOnContract {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_id: types::AccountId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ReceiptsByTxHash {
    #[validate(custom = "crate::errors::validate_crypto_hash")]
    pub transaction_hash: String,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ActionReceiptsByAccountId {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ActionReceiptsByAccountIdOnContract {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_id: types::AccountId,
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
    pub activities: Vec<Activity>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct ActionReceiptsResponse {
    pub activities: Vec<Activity>,
}

// *** Types ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Transaction {
    /// Transaction Hash of the the transction
    pub transaction_hash: types::TransactionHash,
    /// An account on which behalf transaction is signed
    pub signer_account_id: types::AccountId,
    /// An access key which was used to sign the original transaction
    pub signer_public_key: types::PublicKey,
    /// Receiver account for this transaction
    pub receiver_account_id: types::AccountId,
    /// The hash of the block this transaction was included in
    pub block_hash: types::CryptoHash,
    /// A list of actions to be applied
    pub activities: Vec<Activity>,
    /// Timestamp when the transaction was finalized
    pub timestamp: types::U64,
    /// Transaction cost in Yocto Near
    pub total_gas_cost: types::U64,
    /// Amount of Near transferred during this transaction
    pub amount: types::U128,
    /// Status of the Transaction. Finalized | Pending | Failed
    pub status: TxStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub enum TxStatus {
    Pending,
    Failed,
    Success,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Activity {
    /// An unique id for the receipt
    pub receipt_id: types::ReceiptId,
    /// An issuer account_id of a particular receipt.
    /// `predecessor_account_id` could be either `Transaction` `signer_id` or intermediate contract's `account_id`.
    pub predecessor_account_id: types::AccountId,
    /// `receiver_account_id` is a receipt destination.
    pub receiver_account_id: types::AccountId,
    /// `signer_account_id` is the original signer who authorized this transaction
    pub signer_account_id: types::AccountId,
    /// `signer_public_key` is public key the `signer_account_id` used to sign this transaction
    pub signer_public_key: types::PublicKey,
    /// List of operations the signer authorized
    pub operations: Vec<Operation>,
    /// Status of the receipt. Success | Failure | SuccessValue | SuccessReceipt
    pub status: ActivityStatus,
    // Logs produced while executing receipts
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub enum ActivityStatus {
    Pending,
    Failure(String),
    SuccessValue,
    SuccessReceipt, // SuccessActivity
}

// impl Activity {
//  // Converts self.logs to Vec<Events> (might be empty)
//  fn events(&self) -> Vec<Event> {
//      todo!()
//  }
// }

// impl TryFrom<&views::ReceiptView> for Activity {
//  type Error = &'static str;

//  fn try_from(receipt: &views::ReceiptView) -> Result<Self, Self::Error> {
//      if let views::ReceiptEnumView::Action {
//          signer_id,
//          signer_public_key,
//          actions,
//          ..
//      } = &receipt.receipt
//      {
//          Ok(Self {
//              receipt_id: types::CryptoHash(receipt.receipt_id),
//              predecessor_id: types::AccountId(receipt.predecessor_id.clone()),
//              receiver_id: types::AccountId(receipt.receiver_id.clone()),
//              signer_id: types::AccountId(signer_id.clone()),
//              signer_public_key: types::PublicKey(signer_public_key.clone()),
//              operations: actions.iter().map(Into::into).collect(),
//              status: ActivityStatus::Pending,
//              logs: vec!("".to_string()),
//          })
//      } else {
//          Err("Only `ReceiptEnumView::Action` can be converted into Action")
//      }
//  }
// }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub enum Operation {
    CreateAccount,
    DeployContract {
        #[serde(with = "base64_format")]
        code: Vec<u8>,
    },
    FunctionCall {
        method_name: String,
        #[serde(with = "base64_format")]
        args: Vec<u8>,
        gas: types::U64,
        #[serde(with = "u128_dec_format")]
        deposit: u128,
    },
    Transfer {
        #[serde(with = "u128_dec_format")]
        deposit: u128,
    },
    Stake {
        #[serde(with = "u128_dec_format")]
        stake: u128,
        public_key: near_crypto::PublicKey,
    },
    AddKey {
        public_key: near_crypto::PublicKey,
        access_key: views::AccessKeyView,
    },
    DeleteKey {
        public_key: types::U128,
    },
    DeleteAccount {
        beneficiary_id: types::AccountId,
    },
}

// impl From<&views::ActionView> for Operation {
//  fn from(action: &views::ActionView) -> Self {
//      match action {
//          &views::ActionView::CreateAccount => Self::CreateAccount,
//          views::ActionView::DeployContract { code } => {
//              Self::DeployContract { code: code.clone() }
//          }
//          views::ActionView::FunctionCall {
//              method_name,
//              args,
//              gas,
//              deposit,
//          } => Self::FunctionCall {
//              method_name: method_name.to_string(),
//              args: args.clone(),
//              gas: types::numeric::U64(*gas),
//              deposit: *deposit,
//          },
//          views::ActionView::Transfer { deposit } => Self::Transfer { deposit: *deposit },
//          views::ActionView::Stake { stake, public_key } => Self::Stake {
//              stake: *stake,
//              public_key: public_key.clone(),
//          },
//          views::ActionView::AddKey {
//              public_key,
//              access_key,
//          } => Self::AddKey {
//              public_key: public_key.clone(),
//              access_key: access_key.clone(),
//          },
//          views::ActionView::DeleteKey { public_key } => Self::DeleteKey {
//              public_key: public_key.clone(),
//          },
//          views::ActionView::DeleteAccount { beneficiary_id } => Self::DeleteAccount {
//              beneficiary_id: types::account_id::AccountId(beneficiary_id.clone()),
//          },
//      }
//  }
// }
