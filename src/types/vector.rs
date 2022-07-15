use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

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

pub(crate) fn base64_to_string(
    value: &Option<Base64VecU8>,
) -> crate::Result<Option<String>> {
    Ok(if let Some(v) = value {
        Some(serde_json::to_string(&v)?)
    } else {
        None
    })
}