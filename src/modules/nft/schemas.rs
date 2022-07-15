use paperclip::actix::Apiv2Schema;

use crate::types;

// *** Requests ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftCountsRequest {
    pub account_id: types::AccountId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftCollectionRequest {
    pub account_id: types::AccountId,
    pub contract_account_id: types::AccountId,
}

// duplicate in each folder
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct MetadataRequest {
    pub contract_account_id: types::AccountId,
}

/// `token_id` is available at `NftCollectionByContractResponse`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftRequest {
    pub contract_account_id: types::AccountId,
    pub token_id: String,
}

// *** Responses ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftCountsResponse {
    pub nft_counts: Vec<NftCount>,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftsResponse {
    pub nfts: Vec<Nft>,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftResponse {
    pub nft: Nft,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryResponse {
    pub history: Vec<HistoryItem>,
    pub nft: Nft,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct MetadataResponse {
    pub metadata: NftContractMetadata,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

// ---

/// This type describes the history of NFT movements.
/// Note, it's not attached to any user, it's the whole history of NFT movements.
/// `cause` is one of ["mint", "transfer", "burn"]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryItem {
    // todo add status
    pub cause: String,
    pub old_account_id: Option<types::AccountId>,
    pub new_account_id: Option<types::AccountId>,
    // TODO PHASE 2 add index here
    // pub index: types::U128,
    pub block_timestamp_nanos: types::U64,
    pub block_height: types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftCount {
    pub contract_account_id: types::AccountId,
    pub nft_count: u32,
    // TODO PHASE 1 naming.
    pub last_updated_at_timestamp_nanos: types::U128,
    pub contract_metadata: NftContractMetadata,
}

/// The type for Non Fungible Token Contract Metadata. Inspired by
/// https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized data_provider assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<String>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

/// The type for Non Fungible Token. Inspired by
/// https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct Nft {
    pub token_id: String,
    pub owner_account_id: String,
    pub metadata: NftMetadata,
    // TODO PHASE 1 do we want to show them? People often put here weird things
    // pub approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
}

/// The type for Non Fungible Token Metadata. Inspired by
/// https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed data_provider
    pub media_hash: Option<String>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<String>, // ISO 8601 datetime when token was issued or minted
    pub expires_at: Option<String>, // ISO 8601 datetime when token expires
    pub starts_at: Option<String>, // ISO 8601 datetime when token starts being valid
    pub updated_at: Option<String>, // ISO 8601 datetime when token was last updated
    pub extra: Option<String>, // anything extra the NFT wants to data_provider on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<String>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}
