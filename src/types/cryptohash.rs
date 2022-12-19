use std::{fmt, str::FromStr};

use derive_more::{AsRef, Deref, From, FromStr, Into};
use paperclip::v2::{models::DataType, schema::TypedData};
use serde::{Deserialize, Serialize};

#[derive(
    Eq,
    Ord,
    Hash,
    Clone,
    PartialEq,
    PartialOrd,
    From,
    FromStr,
    Into,
    AsRef,
    Deref,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct CryptoHash(pub(crate) near_primitives::hash::CryptoHash);

// pub type TransactionHash = CryptoHash;
// pub type ReceiptId = CryptoHash;

impl fmt::Debug for CryptoHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl TypedData for CryptoHash {
    fn data_type() -> DataType {
        DataType::String
    }
}

pub(crate) fn _extract_cryptohash(
    cryptohash: &str,
) -> crate::Result<Option<near_primitives::hash::CryptoHash>> {
    Ok(if cryptohash.is_empty() {
        None
    } else {
        Some(near_primitives::hash::CryptoHash::from_str(cryptohash)?)
    })
}
