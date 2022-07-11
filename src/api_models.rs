use paperclip::actix::Apiv2Schema;

// Models for the errors are in the separate file src/errors.rs
pub(crate) type Result<T> = std::result::Result<T, crate::errors::Error>;

// *** Requests ***

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BalanceRequest {
    pub account_id: super::types::AccountId,
}

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

/// `token_id` is available at `NftCollectionByContractResponse`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct NftRequest {
    pub contract_account_id: super::types::AccountId,
    pub token_id: String,
}

// *** Responses ***

/// `total_balance` is the sum of `available_balance` and `staked_balance`
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NearBalanceResponse {
    // TODO PHASE 1 naming
    pub total_balance: super::types::U128,
    pub available_balance: super::types::U128,
    pub staked_balance: super::types::U128,
    pub near_metadata: Metadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

/// This response gives the information about all the available balances for the user.
/// The answer gives the list of NEAR, FT balances, could be used for Multi Tokens.
/// For MTs and other standards, balances could have multiple entries for one contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CoinBalancesResponse {
    pub balances: Vec<Coin>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftCollectionOverviewResponse {
    // TODO PHASE 1 naming
    // NFTSometing or NftSomething? I prefer the second one, inspired by https://medium.com/fantageek/using-camelcase-for-abbreviations-232eb67d872
    pub nft_collection_overview: Vec<NftCollectionByContract>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftCollectionByContractResponse {
    // TODO PHASE 1 naming
    pub nft_collection: Vec<NonFungibleToken>,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftResponse {
    pub nft: NonFungibleToken,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NearHistoryResponse {
    pub near_history: Vec<NearHistoryItem>,
    pub near_metadata: Metadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

/// This response provides the history of the coin movements for the given contract.
/// If the contract implements several standards (e.g. FT and MT),
/// `contract_metadata` will give you only the one metadata.
/// We've decided for you, it will be FT metadata.
/// Please do not implement several standards in one contract.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CoinHistoryResponse {
    pub coin_history: Vec<CoinHistoryItem>,
    pub contract_metadata: Metadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftHistoryResponse {
    // TODO PHASE 1 naming. nft_history? history?
    // Metadata: aLso think about MT, there will be also token_metadata. Should we name it coin_metadata? We can use here nft_metadata to avoid interceptions
    pub token_history: Vec<NftHistoryItem>,
    pub token_metadata: NonFungibleToken,
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FtContractMetadataResponse {
    pub contract_metadata: FtContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftContractMetadataResponse {
    pub contract_metadata: NftContractMetadata,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// ---

/// This type describes general coin information.
/// It is used for NEAR, FT balances, could be used for Multi Tokens.
/// `standard`: "nearprotocol" for NEAR, "nep141" for FT.
/// For MTs and other standards, we could have multiple coins for one contract.
/// For NEAR and FTs, coin_metadata contains general metadata (the only available option, though).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Coin {
    pub standard: String,
    // TODO PHASE 1 (idea) it would be great to match this naming with `NearBalanceResponse.total_balance`
    // because we can add here staking info later. This name could always give the answer about total balance
    pub balance: super::types::U128,
    // TODO PHASE 1 if we use `near` here for NEAR, we can make it not null.
    // But, it could feel weird because we restrict use `near` (lowercase) at the endpoints.
    // Maybe redirect people to NEAR on our side + make this not null?
    pub contract_account_id: Option<super::types::AccountId>,
    pub coin_metadata: Metadata,
    // TODO PHASE 1 (idea) I think it would be great to add here the info about last update moment. Timestamp, later also index
    // I'm already doing it at NftCollectionByContract
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NearHistoryItem {
    pub involved_account_id: Option<super::types::AccountId>,
    // TODO PHASE 1 naming. This one is 100% weird, I don't like using it
    pub delta_balance: super::types::I128,
    pub delta_available_balance: super::types::I128,
    pub delta_staked_balance: super::types::I128,
    pub total_balance: super::types::U128,
    pub available_balance: super::types::U128,
    pub staked_balance: super::types::U128,
    // TODO PHASE 1 do we need this field? It's actually debug info for my implementation of balance_changes
    // I really want to provide cause, but not this cause. I want detailed cause (was it a transfer, or fee, or a contract call)
    pub cause: String,
    // TODO PHASE 2 add index here
    // pub index: super::types::U128,
    pub block_timestamp_nanos: super::types::U64,
    // TODO PHASE 2 add this when we have all the data in the same DB. Now we can't join with blocks
    // pub block_height: super::types::U64,
    // TODO PHASE 1 (idea) do we want to add here tx_hash/receipt_id? We may want to add it at many places
}

/// This type describes the history of coin movements for the given user.
/// Coins could be NEAR, FT, it could be also later used for Multi Tokens.
/// `action_kind` is one of ["mint", "transfer", "burn"]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CoinHistoryItem {
    pub action_kind: String,
    pub involved_account_id: Option<super::types::AccountId>,
    // TODO PHASE 1 (idea) it would be great to match this naming with NearHistoryItem fields
    pub delta_balance: super::types::I128,
    pub balance: super::types::U128,
    // TODO PHASE 1 I decided to make it Optional and pass null here for FT. We anyway have contract_metadata on the top
    // It could provide us with problems if we have 2 standards on one contract
    pub coin_metadata: Option<Metadata>,
    // TODO PHASE 2 add index here
    // pub index: super::types::U128,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

/// This type describes the history of NFT movements.
/// Note, it's not attached to any user, it's the whole history of NFT movements.
/// `action_kind` is one of ["mint", "transfer", "burn"]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftHistoryItem {
    pub action_kind: String,
    pub old_account_id: Option<super::types::AccountId>,
    pub new_account_id: Option<super::types::AccountId>,
    // TODO PHASE 2 add index here
    // pub index: super::types::U128,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NftCollectionByContract {
    pub contract_account_id: super::types::AccountId,
    pub nft_count: u32,
    // TODO PHASE 1 naming.
    pub last_updated_at_timestamp_nanos: super::types::U128,
    pub contract_metadata: NftContractMetadata,
}

/// This type describes general Metadata info, collecting the most important fields from different standards in the one format.
/// `decimals` may contain `0` if it's not applicable (e.g. if it's general MT metadata)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
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

/// The type for Non Fungible Token Contract Metadata. Inspired by
/// https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata
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

/// The type for Non Fungible Token. Inspired by
/// https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NonFungibleToken {
    pub token_id: String,
    pub owner_account_id: String,
    pub token_metadata: NftItemMetadata,
    // TODO PHASE 1 do we want to show them? People often put here weird things
    // pub approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
}

/// The type for Non Fungible Token Metadata. Inspired by
/// https://nomicon.io/Standards/Tokens/NonFungibleToken/Metadata
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
    // TODO PHASE 2 add index parameter
    // See thoughts at HistoryPaginationParams
    // pub without_updates_after_index: Option<super::types::U128>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryPaginationParams {
    // pub after_timestamp_nanos: Option<super::types::U64>,
    // pub after_block_height: Option<super::types::U64>,
    // I can, but I decided not to add fields above because people will start using it in production
    // assuming it should give the valid pagination.
    // It won't: we will have issues on the boards because we may have many lines at the same block_height.
    // I want to add fields above to provide the functionality to load the history from the given moment, without knowing the index.
    // But I will add them only at the same moment with the indexes, so that the users can use both mechanisms and paginate properly.
    // TODO PHASE 2 add index parameter
    // pub after_index: Option<super::types::U128>,
    pub limit: Option<u32>,
}
