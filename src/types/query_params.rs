use crate::{errors, types};
use paperclip::actix::Apiv2Schema;

const DEFAULT_PAGE_LIMIT: u32 = 20;
const MAX_PAGE_LIMIT: u32 = 100;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct BlockParams {
    pub block_timestamp_nanos: Option<types::U64>,
    pub block_height: Option<types::U64>,
}

// Designed to use together with BlockParams
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct PaginationParams {
    // TODO PHASE 2 add index parameter
    // pub without_updates_after_index: Option<super::types::U128>,
    /// Maximum available limit 100
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub struct HistoryPaginationParams {
    // pub after_timestamp_nanos: Option<super::types::U64>,
    // pub after_block_height: Option<super::types::U64>,
    // I can, but I decided not to add fields above because people will start using it in production
    // assuming it should give the valid pagination.
    // It won't: we will have issues on the boards because we may have many lines at the same block_height.
    // I want to add fields above to provide the functionality to load the history from the given moment, without knowing the index.
    // But I will add them only at the same moment with the indexes, so that the users can use both mechanisms and paginate properly.
    // TODO PHASE 2 add index parameter
    // pub after_index: Option<super::types::U128>,
    pub limit: Option<u32>,
}

// Helper for parsing the data from user
pub(crate) struct Pagination {
    pub limit: u32,
}

impl From<PaginationParams> for Pagination {
    fn from(params: PaginationParams) -> Self {
        Self {
            limit: params.limit.unwrap_or(DEFAULT_PAGE_LIMIT),
        }
    }
}

impl From<HistoryPaginationParams> for Pagination {
    fn from(params: HistoryPaginationParams) -> Self {
        Self {
            limit: params.limit.unwrap_or(DEFAULT_PAGE_LIMIT),
        }
    }
}

pub(crate) struct HistoryPagination {
    // start_after. Not including this!
    pub block_height: u64,
    pub block_timestamp: u64,
    // TODO PHASE 2 add index parameter
    // pub index: u128,
    pub limit: u32,
}

pub(crate) fn check_block_params(params: &BlockParams) -> crate::Result<()> {
    if params.block_height.is_some() && params.block_timestamp_nanos.is_some() {
        Err(errors::ErrorKind::InvalidInput(
            "Both block_height and block_timestamp_nanos found. Please provide only one of values"
                .to_string(),
        )
        .into())
    } else {
        Ok(())
    }
}

pub(crate) fn check_limit(limit_param: Option<u32>) -> crate::Result<()> {
    if let Some(limit) = limit_param {
        if limit > MAX_PAGE_LIMIT || limit == 0 {
            return Err(errors::ErrorKind::InvalidInput(format!(
                "Limit should be in range [1, {}]",
                MAX_PAGE_LIMIT
            ))
            .into());
        }
    }
    Ok(())
}
