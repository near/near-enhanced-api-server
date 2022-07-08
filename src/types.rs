use std::fmt;
use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use paperclip::v2::{models::DataType, schema::TypedData};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{api_models, db_models, errors, utils};

#[derive(
    Eq,
    Ord,
    Hash,
    Clone,
    PartialEq,
    PartialOrd,
    derive_more::From,
    derive_more::Into,
    derive_more::AsRef,
    derive_more::Deref,
    derive_more::FromStr,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(transparent)]
pub struct AccountId(pub(crate) near_primitives::types::AccountId);

impl fmt::Debug for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl TypedData for AccountId {
    fn data_type() -> DataType {
        DataType::String
    }
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-sdk/src/json_types/integers.rs
macro_rules! impl_str_type {
    ($iden: ident, $ty: tt) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, BorshDeserialize, BorshSerialize,
        )]
        pub struct $iden(pub $ty);

        impl From<$ty> for $iden {
            fn from(v: $ty) -> Self {
                Self(v)
            }
        }

        impl From<$iden> for $ty {
            fn from(v: $iden) -> $ty {
                v.0
            }
        }

        impl Serialize for $iden {
            fn serialize<S>(
                &self,
                serializer: S,
            ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $iden {
            fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
            where
                D: Deserializer<'de>,
            {
                let s: String = Deserialize::deserialize(deserializer)?;
                Ok(Self(str::parse::<$ty>(&s).map_err(|err| {
                    serde::de::Error::custom(err.to_string())
                })?))
            }
        }

        impl TypedData for $iden {
            fn data_type() -> DataType {
                DataType::String
            }
        }
    };
}

impl_str_type!(U128, u128);
impl_str_type!(U64, u64);
impl_str_type!(I128, i128);

// Helper for parsing the data collected from DB
pub struct Block {
    pub timestamp: u64,
    pub height: u64,
}

impl TryFrom<&db_models::Block> for Block {
    type Error = errors::Error;

    fn try_from(block: &db_models::Block) -> api_models::Result<Self> {
        Ok(Self {
            timestamp: utils::to_u64(&block.block_timestamp)?,
            height: utils::to_u64(&block.block_height)?,
        })
    }
}

// Helper for parsing the data from user
pub struct CoinBalancesPagination {
    pub limit: u32,
}

impl From<api_models::BalancesPaginationParams> for CoinBalancesPagination {
    fn from(params: api_models::BalancesPaginationParams) -> Self {
        Self {
            limit: params.limit.unwrap_or(crate::DEFAULT_PAGE_LIMIT),
        }
    }
}

pub struct HistoryPagination {
    // start_after. Not including this!
    pub block_height: u64,
    pub block_timestamp: u64,
    // TODO PHASE 2 add index parameter
    // pub index: u128,
    pub limit: u32,
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-sdk/src/json_types/vector.rs
/// Helper class to serialize/deserialize `Vec<u8>` to base64 string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
pub struct Base64VecU8(pub Vec<u8>);

impl From<Vec<u8>> for Base64VecU8 {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<Base64VecU8> for Vec<u8> {
    fn from(v: Base64VecU8) -> Vec<u8> {
        v.0
    }
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/fungible_token/metadata.rs
#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
pub struct FungibleTokenMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
    pub decimals: u8,
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/metadata.rs
/// Metadata for the NFT contract itself.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NFTContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

/// Metadata on the individual token level.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, BorshDeserialize, BorshSerialize)]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<String>, // ISO 8601 datetime when token was issued or minted
    pub expires_at: Option<String>, // ISO 8601 datetime when token expires
    pub starts_at: Option<String>, // ISO 8601 datetime when token starts being valid
    pub updated_at: Option<String>, // ISO 8601 datetime when token was last updated
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/token.rs
/// Note that token IDs for NFTs are strings on NEAR. It's still fine to use autoincrementing numbers as unique IDs if desired, but they should be stringified. This is to make IDs more future-proof as chain-agnostic conventions and standards arise, and allows for more flexibility with considerations like bridging NFTs across chains, etc.
pub type TokenId = String;

/// In this implementation, the Token struct takes two extensions standards (metadata and approval) as optional fields, as they are frequently used in modern NFTs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: Option<TokenMetadata>,
    pub approved_account_ids: Option<std::collections::HashMap<AccountId, u64>>,
}

impl From<api_models::FtContractMetadata> for api_models::Metadata {
    fn from(metadata: api_models::FtContractMetadata) -> Self {
        api_models::Metadata {
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            decimals: metadata.decimals,
        }
    }
}

impl From<api_models::NearBalanceResponse> for api_models::Coin {
    fn from(near_coin: api_models::NearBalanceResponse) -> Self {
        api_models::Coin {
            standard: "nearprotocol".to_string(),
            balance: near_coin.total_balance,
            contract_account_id: None,
            coin_metadata: near_coin.near_metadata,
        }
    }
}

impl TryFrom<db_models::NearHistoryInfo> for api_models::NearHistoryItem {
    type Error = errors::Error;

    fn try_from(info: db_models::NearHistoryInfo) -> api_models::Result<Self> {
        let involved_account_id: Option<AccountId> =
            if let Some(account_id) = info.involved_account_id {
                Some(near_primitives::types::AccountId::from_str(&account_id)?.into())
            } else {
                None
            };
        Ok(Self {
            involved_account_id,
            delta_balance: utils::to_i128(&info.delta_balance)?.into(),
            delta_available_balance: utils::to_i128(&info.delta_available_balance)?.into(),
            delta_staked_balance: utils::to_i128(&info.delta_staked_balance)?.into(),
            total_balance: utils::to_u128(&info.total_balance)?.into(),
            available_balance: utils::to_u128(&info.available_balance)?.into(),
            staked_balance: utils::to_u128(&info.staked_balance)?.into(),
            cause: info.cause,
            block_timestamp_nanos: utils::to_u64(&info.block_timestamp_nanos)?.into(),
        })
    }
}

impl TryFrom<db_models::NftHistoryInfo> for api_models::NftHistoryItem {
    type Error = errors::Error;

    fn try_from(info: db_models::NftHistoryInfo) -> api_models::Result<Self> {
        Ok(Self {
            action_kind: info.action_kind,
            old_account_id: utils::extract_account_id(&info.old_account_id)?
                .map(|account| account.into()),
            new_account_id: utils::extract_account_id(&info.new_account_id)?
                .map(|account| account.into()),
            block_timestamp_nanos: utils::to_u64(&info.block_timestamp_nanos)?.into(),
            block_height: utils::to_u64(&info.block_height)?.into(),
        })
    }
}

impl TryFrom<NFTContractMetadata> for api_models::NftContractMetadata {
    type Error = errors::Error;

    fn try_from(metadata: NFTContractMetadata) -> api_models::Result<Self> {
        Ok(Self {
            spec: metadata.spec,
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            base_uri: metadata.base_uri,
            reference: metadata.reference,
            reference_hash: utils::base64_to_string(&metadata.reference_hash)?,
        })
    }
}

impl TryFrom<Token> for api_models::NonFungibleToken {
    type Error = errors::Error;

    fn try_from(token: Token) -> api_models::Result<Self> {
        let metadata = token.metadata.ok_or_else(|| {
            errors::ErrorKind::RPCError("NFT requires to have metadata filled".to_string())
        })?;

        Ok(Self {
            token_id: token.token_id,
            owner_account_id: token.owner_id.0.to_string(),
            token_metadata: api_models::NftItemMetadata {
                title: metadata.title,
                description: metadata.description,
                media: metadata.media,
                media_hash: utils::base64_to_string(&metadata.media_hash)?,
                copies: metadata.copies,
                issued_at: metadata.issued_at,
                expires_at: metadata.expires_at,
                starts_at: metadata.starts_at,
                updated_at: metadata.updated_at,
                extra: metadata.extra,
                reference: metadata.reference,
                reference_hash: utils::base64_to_string(&metadata.reference_hash)?,
            },
        })
    }
}

// temp solution to pass 2 different connection pools
pub struct DBWrapper {
    pub pool: sqlx::Pool<sqlx::Postgres>,
}
