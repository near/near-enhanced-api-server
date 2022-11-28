use std::fmt;

use derive_more::{AsRef, Deref, From, FromStr, Into};
use paperclip::v2::{models::DataType, schema::TypedData};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use near_primitives::hash::CryptoHash;

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
pub struct TransactionHash(String);

impl TransactionHash {
    pub fn to_crypto_hash(self) -> Result<CryptoHash,String> {
        let hash = CryptoHash::from_str(&self);
       if let Err(error) = hash {
         return Err(error.to_string())
       }
        Ok(hash.unwrap())
    }
}

impl fmt::Debug for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}
impl TypedData for TransactionHash {
    fn data_type() -> DataType {
        DataType::String
    }
}
