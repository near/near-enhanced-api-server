use derive_more::{AsRef, Deref, From, FromStr, Into};
use near_primitives::hash::CryptoHash;
use paperclip::v2::{models::DataType, schema::TypedData};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(
    Eq,
    Ord,
    Hash,
    Clone,
    PartialEq,
    PartialOrd,
    From,
    Into,
    AsRef,
    Deref,
    FromStr,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct ReceiptId(String);

impl ReceiptId {
    pub fn to_crypto_hash(self) -> Result<CryptoHash, String> {
        let hash = CryptoHash::from_str(&self);
        if let Err(error) = hash {
            return Err(error.to_string());
        }
        Ok(hash.unwrap())
    }
}
impl fmt::Debug for ReceiptId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
impl TypedData for ReceiptId {
    fn data_type() -> DataType {
        DataType::String
    }
}
