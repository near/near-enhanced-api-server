use paperclip::actix::Apiv2Schema;

use crate::types;

// *** Requests ***

// move to coins
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceRequest {
    pub account_id: types::AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceByContractRequest {
    pub account_id: types::AccountId,
    pub contract_account_id: types::AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceHistoryRequest {
    pub account_id: types::AccountId,
    pub contract_account_id: types::AccountId,
}

// duplicate in each folder
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct ContractMetadataRequest {
    pub contract_account_id: types::AccountId,
}

// *** Responses ***

/// `total_balance` is the sum of `available_balance` and `staked_balance`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NearBalanceResponse {
    // TODO PHASE 1 naming
    pub total_balance: types::U128,
    pub available_balance: types::U128,
    pub staked_balance: types::U128,
    pub near_metadata: Metadata,
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NearHistoryResponse {
    // TODO PHASE 1 this interface should be as CoinHistoryItem
    pub coin_history: Vec<NearHistoryItem>,
    // TODO PHASE 1 do we need to serve contract metadata at history at all?
    pub contract_metadata: Metadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

/// This response provides the history of the coin movements for the given contract.
/// If the contract implements several standards (e.g. FT and MT),
/// `contract_metadata` will give you only the one metadata.
/// We've decided for you, it will be FT metadata.
/// Please do not implement several standards in one contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct CoinHistoryResponse {
    pub coin_history: Vec<CoinHistoryItem>,
    pub contract_metadata: Metadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct FtContractMetadataResponse {
    pub contract_metadata: FtContractMetadata,
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
    // TODO PHASE 1 (idea) it would be great to match this naming with `NearBalanceResponse.total_balance`
    // because we can add here staking info later. This name could always give the answer about total balance
    pub balance: types::U128,
    /// null for NEAR, not null otherwise
    pub contract_account_id: Option<types::AccountId>,
    pub coin_metadata: Metadata,
    // TODO PHASE 1 (idea) I think it would be great to add here the info about last update moment. Timestamp, later also index
    // I'm already doing it at NftCollectionByContract
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NearHistoryItem {
    pub involved_account_id: Option<types::AccountId>,
    // TODO PHASE 1 naming. This one is 100% weird, I don't like using it
    pub delta_balance: types::I128,
    pub delta_available_balance: types::I128,
    pub delta_staked_balance: types::I128,
    pub total_balance: types::U128,
    pub available_balance: types::U128,
    pub staked_balance: types::U128,
    // TODO PHASE 1 do we need this field? It's actually debug info for my implementation of balance_changes
    // I really want to provide cause, but not this cause. I want detailed cause (was it a transfer, or fee, or a contract call)
    pub cause: String,
    // TODO PHASE 2 add index here
    // pub index: types::U128,
    pub block_timestamp_nanos: types::U64,
    // TODO PHASE 2 add this when we have all the data in the same DB. Now we can't join with blocks
    // pub block_height: types::U64,
    // TODO PHASE 1 (idea) do we want to add here tx_hash/receipt_id? We may want to add it at many places
}

/// This type describes the history of coin movements for the given user.
/// Coins could be NEAR, FT, it could be also later used for Multi Tokens.
/// `action_kind` is one of ["mint", "transfer", "burn"]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct CoinHistoryItem {
    pub action_kind: String,
    pub involved_account_id: Option<types::AccountId>,
    // TODO PHASE 1 (idea) it would be great to match this naming with NearHistoryItem fields
    pub delta_balance: types::I128,
    pub balance: types::U128,
    // TODO PHASE 1 I decided to make it Optional and pass null here for FT. We anyway have contract_metadata on the top
    // It could provide us with problems if we have 2 standards on one contract
    pub coin_metadata: Option<Metadata>,
    // TODO PHASE 2 add index here
    // pub index: types::U128,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

/// This type describes general Metadata info, collecting the most important fields from different standards in the one format.
/// `decimals` may contain `0` if it's not applicable (e.g. if it's general MT metadata)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Metadata {
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    // TODO PHASE 1 discuss and check the doc here
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
