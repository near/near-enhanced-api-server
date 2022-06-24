use paperclip::actix::Apiv2Schema;

// Models for the errors are in the separate file src/errors.rs
pub(crate) type Result<T> = std::result::Result<T, crate::errors::Error>;

// todo docs for all the models
// todo read about using enums here

// *** Requests ***

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

// *** Responses ***

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct BalancesResponse {
    pub balances: Vec<Coin>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NearBalanceResponse {
    pub balance: super::types::U128,
    pub available: super::types::U128, // todo naming
    pub staked: super::types::U128,
    pub metadata: CoinMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct HistoryResponse {
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct MtMetadataResponse {
    pub metadata: MtContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

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
    pub nfts: Vec<NftItemMetadata>,
    pub metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// ---

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct Coin {
    pub standard: String,
    pub balance: super::types::U128,
    pub contract_account_id: Option<super::types::AccountId>,
    pub metadata: CoinMetadata,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct HistoryInfo {
    pub action_kind: String, // mint transfer burn
    pub involved_account_id: Option<super::types::AccountId>,
    pub delta_balance: super::types::I128,
    pub balance: super::types::U128,
    pub metadata: CoinMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct NftsByContractInfo {
    pub contract_account_id: super::types::AccountId,
    pub nft_count: u32,
    pub metadata: NftContractMetadata,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct CoinMetadata {
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub decimals: u8,
}

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct MtContractMetadata {
    pub spec: String,
    pub name: String,
    //     todo
}

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
pub(crate) struct QueryParams {
    pub block_timestamp_nanos: Option<super::types::U64>,
    pub block_height: Option<super::types::U64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct PaginatedQueryParams {
    // todo die if both are given
    pub block_timestamp_nanos: Option<super::types::U64>,
    pub block_height: Option<super::types::U64>,
    pub page: Option<u32>,
}
