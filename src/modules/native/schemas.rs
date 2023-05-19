use paperclip::actix::Apiv2Schema;
use validator::Validate;

use crate::types;

// *** Requests ***

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct BalanceRequest {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
}

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct HistoryRequest {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_account_id: types::AccountId,
}

// *** Responses ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NearBalanceResponse {
    pub balance: NearBalance,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NearBalance {
    /// Sum of staked and nonstaked balances
    pub amount: types::U128,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NearHistoryResponse {
    pub history: Vec<HistoryItem>,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

/// This type describes the history of the operations (NEAR, FT) for the given user.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryItem {
    pub event_index: types::U128,
    pub involved_account_id: Option<types::AccountId>,
    pub delta_balance: String,
    pub balance: types::U128,
    pub cause: String,
    pub status: String,
    pub metadata: Metadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

/// This type describes general Metadata info
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub decimals: u8,
}
