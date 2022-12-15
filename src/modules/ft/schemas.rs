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
pub struct BalanceByContractRequest {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub account_id: types::AccountId,
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_account_id: types::AccountId,
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

#[derive(
    Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema,
)]
pub struct ContractMetadataRequest {
    #[validate(custom = "crate::errors::validate_account_id")]
    pub contract_account_id: types::AccountId,
}

// *** Responses ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct FtBalanceByContractResponse {
    pub balance: FtBalance,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct FtBalancesResponse {
    pub balances: Vec<FtBalance>,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct FtBalance {
    pub amount: types::U128,
    pub contract_account_id: types::AccountId,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryResponse {
    pub history: Vec<HistoryItem>,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct FtContractMetadataResponse {
    pub metadata: FtContractMetadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

// ---

/// This type describes the history of the operations (NEAR, FT) for the given user.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryItem {
    // TODO PHASE 2 add index here
    // pub index: types::U128,
    pub involved_account_id: Option<types::AccountId>,
    pub delta_balance: types::I128,
    pub balance: types::U128,
    pub cause: String,
    pub status: String,
    pub metadata: Metadata,
    pub block_timestamp_nanos: types::U64,
    // TODO PHASE 2 add this when we have all the data in the same DB. Now we can't join with blocks
    // pub block_height: types::U64,
}

/// This type describes general Metadata info, collecting the most important fields from different standards in the one format.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub decimals: u8,
}

/// The type for FT Contract Metadata. Inspired by
/// https://nomicon.io/Standards/Tokens/FungibleToken/Metadata
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct FtContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<String>,
    pub decimals: u8,
}
