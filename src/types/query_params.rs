use crate::{errors, types};
use paperclip::actix::Apiv2Schema;

const DEFAULT_PAGE_LIMIT: u32 = 20;
const MAX_PAGE_LIMIT: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BlockParams {
    pub block_timestamp_nanos: Option<types::U64>,
    pub block_height: Option<types::U64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct PaginationParams {
    /// Maximum available limit 100
    pub limit: Option<u32>,
    pub after_event_index: Option<types::U128>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct LimitParams {
    pub limit: Option<u32>,
}

// Helper for parsing the data from user
#[derive(Debug)]
pub(crate) struct Pagination {
    pub limit: u32,
    pub after_event_index: Option<u128>,
}

pub(crate) fn checked_get_limit(limit_param: Option<u32>) -> crate::Result<u32> {
    Ok(if let Some(limit) = limit_param {
        if limit > MAX_PAGE_LIMIT || limit == 0 {
            return Err(errors::ErrorKind::InvalidInput(format!(
                "Limit should be in range [1, {}]",
                MAX_PAGE_LIMIT
            ))
            .into());
        }
        limit
    } else {
        DEFAULT_PAGE_LIMIT
    })
}
