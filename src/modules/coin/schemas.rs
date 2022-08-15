use paperclip::actix::Apiv2Schema;
use validator::{Validate, ValidationError};

use crate::types;

// *** Requests ***

// move to coins
#[derive(Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceRequest {
    #[validate(custom = "near_primitives::types::AccountId::validate")]
    pub account_id: types::AccountId,
}

#[derive(Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceByContractRequest {
    #[validate(custom = "near_primitives::types::AccountId::validate")]
    pub account_id: types::AccountId,
    #[validate(custom = "near_primitives::types::AccountId::validate")]
    pub contract_account_id: types::AccountId,
}

#[derive(Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryRequest {
    #[validate(custom = "near_primitives::types::AccountId::validate")]
    pub account_id: types::AccountId,
    pub contract_account_id: types::AccountId,
}

// duplicate in each folder
#[derive(Validate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct ContractMetadataRequest {
    #[validate(custom = "near_primitives::types::AccountId::validate")]
    pub contract_account_id: types::AccountId,
}

// *** Responses ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NearBalanceResponse {
    /// Sum of staked and nonstaked balances
    pub balance: types::U128,
    pub metadata: CoinMetadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

/// This response gives the information about all the available balances for the user.
/// The answer gives the list of NEAR, FT balances, could be used for Multi Tokens.
/// For MTs and other standards, balances could have multiple entries for one contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct CoinBalancesResponse {
    pub balances: Vec<Coin>,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

/// This response provides the coin history (NEAR or by contract).
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

/// This type describes general coin information.
/// It is used for NEAR, FT balances, could be used for Multi Tokens.
///
/// For MTs and other standards, we could have multiple coins for one contract.
/// For NEAR and FTs, coin_metadata contains general metadata (the only available option, though).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Coin {
    /// "nearprotocol" for NEAR, "nep141" for FT
    pub standard: String,
    pub balance: types::U128,
    /// null for NEAR, not null otherwise
    pub contract_account_id: Option<types::AccountId>,
    pub metadata: CoinMetadata,
    // TODO PHASE 1 (idea) I think it would be great to add here the info about last update moment. Timestamp, later also index
    // I'm already doing it at NftCount
}

/// This type describes the history of coin movements for the given user.
/// Coins could be NEAR, FT, it could be also later used for Multi Tokens.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryItem {
    // TODO PHASE 2 add index here
    // pub index: types::U128,
    // TODO PHASE 1 (idea) do we want to add here tx_hash/receipt_id? We may want to add it at many places
    pub involved_account_id: Option<types::AccountId>,
    pub delta_balance: types::I128,
    pub balance: types::U128,
    pub cause: String,
    pub status: String,
    pub coin_metadata: CoinMetadata,
    pub block_timestamp_nanos: types::U64,
    // TODO PHASE 2 add this when we have all the data in the same DB. Now we can't join with blocks
    // pub block_height: types::U64,
}

/// This type describes general Metadata info, collecting the most important fields from different standards in the one format.
/// `decimals` may contain `0` if it's not applicable (e.g. if it's general MT metadata)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct CoinMetadata {
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

pub fn validate(account_id: &str) -> Result<(), ValidationError> {
    Err(ValidationError::new("something"))
}
