use paperclip::actix::Apiv2Schema;

// Models for the errors are in the separate file src/errors.rs
pub(crate) type Result<T> = std::result::Result<T, crate::errors::Error>;

// todo docs for all the models
// todo read about using enums here

// *** Requests ***

/// Some comment here
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct BalanceRequest {
    pub account_id: super::types::AccountId,
}

/// Includes Native, FT, later will include MT
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct BalanceByContractRequest {
    pub account_id: super::types::AccountId,
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct BalanceHistoryRequest {
    pub account_id: super::types::AccountId,
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct ContractMetadataRequest {
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct NftItemRequest {
    pub contract_account_id: super::types::AccountId,
    pub token_id: String,
}

// *** Responses ***

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct BalancesResponse {
    // todo remember here could be mt
    pub balances: Vec<Coin>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NearBalanceResponse {
    pub total_balance: super::types::U128,
    pub available_balance: super::types::U128, // todo naming
    pub staked_balance: super::types::U128,
    pub metadata: CoinMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct HistoryResponse {
    // todo remember here could be mt
    pub history: Vec<HistoryInfo>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct FtMetadataResponse {
    pub metadata: FtContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
// #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
// pub(crate) struct MtMetadataResponse {
//     pub metadata: MtContractMetadata,
//     pub block_timestamp_nanos: super::types::U64,
//     pub block_height: super::types::U64,
// }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftMetadataResponse {
    pub metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftCountResponse {
    pub nft_count: Vec<NftsByContractInfo>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftBalanceResponse {
    pub nfts: Vec<NonFungibleToken>,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftHistoryResponse {
    pub history: Vec<NftHistoryInfo>,
    pub token_metadata: NonFungibleToken,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

/// Some comment here
///
/// With the details
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NearHistoryResponse {
    pub history: Vec<NearHistoryInfo>,
    pub metadata: CoinMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

/// Some comment here
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftItemResponse {
    pub nft: NonFungibleToken,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// ---

/// Some comment here
///
/// With the details
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct Coin {
    // todo remember here could be mt
    pub standard: String,
    pub balance: super::types::U128,
    pub contract_account_id: Option<super::types::AccountId>,
    pub metadata: CoinMetadata,
    // last_updated_at? index?
}

// todo do we want to give the info about failed tx?
// todo + tx hash/receipt id for all the data
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NearHistoryInfo {
    pub involved_account_id: Option<super::types::AccountId>,
    pub delta_balance: super::types::I128,
    pub delta_available_balance: super::types::I128,
    pub delta_staked_balance: super::types::I128,
    pub total_balance: super::types::U128,
    pub available_balance: super::types::U128, // todo naming
    pub staked_balance: super::types::U128,
    pub cause: String,
    pub index: super::types::U128, // todo naming
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct HistoryInfo {
    // todo remember here could be mt
    pub action_kind: String, // mint transfer burn
    pub involved_account_id: Option<super::types::AccountId>,
    pub delta_balance: super::types::I128,
    pub balance: super::types::U128,
    pub metadata: CoinMetadata,
    pub index: super::types::U128, // todo naming
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// todo I feel it's better to provide receipt id/tx hash here, do we want to add it here?
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftHistoryInfo {
    pub action_kind: String, // mint transfer burn
    pub old_account_id: Option<super::types::AccountId>,
    pub new_account_id: Option<super::types::AccountId>,
    // pub index: super::types::U128, // todo naming. Not implemented yet
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftsByContractInfo {
    pub contract_account_id: super::types::AccountId,
    pub nft_count: u32,
    pub last_updated_at_timestamp: super::types::U128,
    pub metadata: NftContractMetadata,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct CoinMetadata {
    // todo remember here could be mt
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub decimals: u8,
}

/// Some comment here
///
/// With the details
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct FtContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<String>,
    pub decimals: u8,
}

// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
// #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
// pub(crate) struct MtContractMetadata {
//     pub spec: String,
//     pub name: String,
//     //     todo
// }

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<String>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NonFungibleToken {
    pub token_id: String,
    pub owner_account_id: String,
    pub metadata: Option<NftItemMetadata>,
    // todo do we want to show it?
    // pub approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftItemMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<String>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<String>, // ISO 8601 datetime when token was issued or minted
    pub expires_at: Option<String>, // ISO 8601 datetime when token expires
    pub starts_at: Option<String>, // ISO 8601 datetime when token starts being valid
    pub updated_at: Option<String>, // ISO 8601 datetime when token was last updated
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<String>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

// ---

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct BlockParams {
    pub block_timestamp_nanos: Option<super::types::U64>,
    pub block_height: Option<super::types::U64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct CoinBalancesPaginationParams {
    // pub start_after_standard: Option<String>, // todo naming
    // pub start_after_contract_account_id: Option<String>, // todo will not work for MT when we have several tokens for 1 contract
    // pub last_symbol: Option<String>, // todo for mt, we have token_id and symbol, and potentially we can have 1 symbol for different token_ids
    // pub start_after_coin_id: Option<String>, // in reality, we store here token_id for MTs and symbol for FTs
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct NftOverviewPaginationParams {
    pub with_no_updates_after_timestamp_nanos: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct NftBalancePaginationParams {
    // pub start_after_token_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct HistoryPaginationParams {
    // todo if both block_params and last_index_from_previous_page is presented, we should fail
    // pub start_after_index: Option<super::types::U128>, // todo not implemented yet
    pub limit: Option<u32>,
}
