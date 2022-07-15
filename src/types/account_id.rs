use std::fmt;
use std::str::FromStr;

use derive_more::{From, Into, AsRef, Deref, FromStr};
use serde::{Deserialize, Serialize};
use paperclip::v2::{models::DataType, schema::TypedData};

#[derive(Eq, Ord, Hash, Clone, PartialEq, PartialOrd, From, Into, AsRef, Deref, FromStr, Serialize, Deserialize)]
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

pub(crate) fn extract_account_id(
    account_id: &str,
) -> crate::Result<Option<near_primitives::types::AccountId>> {
    if account_id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(near_primitives::types::AccountId::from_str(
            account_id,
        )?))
    }
}
