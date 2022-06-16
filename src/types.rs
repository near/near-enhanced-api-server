use std::fmt;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

use paperclip::v2::{models::DataType, schema::TypedData};
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
    };
}

impl_str_type!(U128, u128);
impl_str_type!(U64, u64);

impl TypedData for U64 {
    fn data_type() -> DataType {
        DataType::String
    }
}

impl TypedData for U128 {
    fn data_type() -> DataType {
        DataType::String
    }
}
