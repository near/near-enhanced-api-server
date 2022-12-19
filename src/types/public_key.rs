use std::fmt;

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
    Into,
    AsRef,
    Deref,
    FromStr,
    Serialize,
    Deserialize,
)]
#[serde(transparent)]
pub struct PublicKey(pub(crate) near_crypto::PublicKey);

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl TypedData for PublicKey {
    fn data_type() -> DataType {
        DataType::String
    }
}
