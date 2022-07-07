use paperclip::actix::Apiv2Schema;

// Models for the errors are in the separate file src/errors.rs
pub(crate) type Result<T> = std::result::Result<T, crate::errors::Error>;

// todo docs for all the models
// todo read about using enums here

// *** Requests ***

/// Some comment here
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceRequest {
    pub account_id: super::types::AccountId,
}

/// Includes Native, FT, later will include MT
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceByContractRequest {
    pub account_id: super::types::AccountId,
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceHistoryRequest {
    pub account_id: super::types::AccountId,
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct ContractMetadataRequest {
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftItemRequest {
    pub contract_account_id: super::types::AccountId,
    pub token_id: String,
}

// *** Responses ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct BalancesResponse {
    // todo remember here could be mt
    pub balances: Vec<Coin>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NearBalanceResponse {
    pub total_balance: super::types::U128,
    pub available_balance: super::types::U128, // todo naming
    pub staked_balance: super::types::U128,
    pub metadata: CoinMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct HistoryResponse {
    // todo remember here could be mt
    pub history: Vec<CoinHistoryInfo>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FtMetadataResponse {
    pub metadata: FtContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
// #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
// pub struct MtMetadataResponse {
//     pub metadata: MtContractMetadata,
//     pub block_timestamp_nanos: super::types::U64,
//     pub block_height: super::types::U64,
// }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftMetadataResponse {
    pub metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftCountResponse {
    pub nft_count: Vec<NftsByContractInfo>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftBalanceResponse {
    pub nfts: Vec<NonFungibleToken>,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftHistoryResponse {
    pub history: Vec<NftHistoryInfo>,
    pub token_metadata: NonFungibleToken,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

/// Some comment here
///
/// With the details
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NearHistoryResponse {
    pub history: Vec<NearHistoryInfo>,
    pub metadata: CoinMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

/// Some comment here
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftItemResponse {
    pub nft: NonFungibleToken,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// ---

/// Some comment here
///
/// With the details
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Coin {
    // todo remember here could be mt
    pub standard: String,
    pub balance: super::types::U128,
    pub contract_account_id: Option<super::types::AccountId>,
    pub metadata: CoinMetadata,
    // last_updated_at? index?
}

// todo + tx hash/receipt id for all the data
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NearHistoryInfo {
    pub involved_account_id: Option<super::types::AccountId>,
    pub delta_balance: super::types::I128,
    pub delta_available_balance: super::types::I128,
    pub delta_staked_balance: super::types::I128,
    pub total_balance: super::types::U128,
    pub available_balance: super::types::U128, // todo naming
    pub staked_balance: super::types::U128,
    pub cause: String,
    // pub index: super::types::U128, // todo naming. Not implemented yet
    pub block_timestamp_nanos: super::types::U64,
    // pub block_height: super::types::U64, // todo add this when we have all the data in the same DB
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CoinHistoryInfo {
    // todo remember here could be mt
    pub action_kind: String, // mint transfer burn
    pub involved_account_id: Option<super::types::AccountId>,
    pub delta_balance: super::types::I128,
    pub balance: super::types::U128,
    pub metadata: CoinMetadata,
    // pub index: super::types::U128, // todo naming
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// todo I feel it's better to provide receipt id/tx hash here, do we want to add it here?
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftHistoryInfo {
    pub action_kind: String, // mint transfer burn
    pub old_account_id: Option<super::types::AccountId>,
    pub new_account_id: Option<super::types::AccountId>,
    // pub index: super::types::U128, // todo naming. Not implemented yet
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftsByContractInfo {
    pub contract_account_id: super::types::AccountId,
    pub nft_count: u32,
    pub last_updated_at_timestamp: super::types::U128,
    pub metadata: NftContractMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CoinMetadata {
    // todo remember here could be mt
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub decimals: u8,
}

/// Some comment here
///
/// With the details
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FtContractMetadata {
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
// pub struct MtContractMetadata {
//     pub spec: String,
//     pub name: String,
//     //     todo
// }

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<String>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NonFungibleToken {
    pub token_id: String,
    pub owner_account_id: String,
    pub metadata: Option<NftItemMetadata>,
    // todo do we want to show it?
    // pub approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftItemMetadata {
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BlockParams {
    pub block_timestamp_nanos: Option<super::types::U64>,
    pub block_height: Option<super::types::U64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalancesPaginationParams {
    // TODO PHASE 1 make the decision about naming
    // TODO PHASE 2 add index parameter
    // also see thoughts at HistoryPaginationParams
    // pub without_updates_after_index: Option<super::types::U128>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryPaginationParams {
    // I decided not to add these fields now because people will start using it in production assuming
    // it should give the valid pagination. It won't: we will have issues on the boards because we can have many lines at the same block_height.
    // I want to add this to provide the functionality to load the history from the given moment, without knowing the index.
    // But we will add this only at the same moment with the indexes, so that the users can use both mechanisms and paginate properly.
    // pub after_timestamp_nanos: Option<super::types::U64>,
    // pub after_block_height: Option<super::types::U64>,
    // TODO PHASE 2 add index parameter
    // pub after_index: Option<super::types::U128>,
    pub limit: Option<u32>,
}
