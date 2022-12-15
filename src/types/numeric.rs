use borsh::{BorshDeserialize, BorshSerialize};
use num_traits::cast::ToPrimitive;
use paperclip::v2::{models::DataType, schema::TypedData};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{errors, BigDecimal};

pub(crate) fn to_u128(x: &BigDecimal) -> crate::Result<u128> {
    x.to_string().parse().map_err(|e| {
        errors::ErrorKind::InternalError(format!("Failed to parse u128 {}: {}", x, e)).into()
    })
}

pub(crate) fn to_u64(x: &BigDecimal) -> crate::Result<u64> {
    x.to_u64().ok_or_else(|| {
        errors::ErrorKind::InternalError(format!("Failed to parse u64 {}", x)).into()
    })
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
