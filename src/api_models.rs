use paperclip::actix::Apiv2Schema;

// Models for the errors are in the separate file src/errors.rs
pub(crate) type Result<T> = std::result::Result<T, crate::errors::Error>;

// todo docs for all the models

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct BalanceRequest {
    pub account_id: super::types::AccountId,
}

/// Includes Native, FT, later will include MT
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct BalanceRequestForContract {
    pub account_id: super::types::AccountId,
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct BalanceResponse {
    // todo we don't have this for FTs
    // go to metadata each time
    pub balances: Vec<CoinInfo>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct CoinInfo {
    // todo use enums here?
    pub standard: String,
    pub token_id: String,
    pub contract_account_id: Option<String>,
    pub amount: super::types::U128,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct QueryParams {
    pub block_timestamp_nanos: Option<super::types::U64>,
    pub block_height: Option<super::types::U64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct PaginatedQueryParams {
    // todo die if both are given
    pub block_timestamp_nanos: Option<super::types::U64>,
    pub block_height: Option<super::types::U64>,
    pub page: Option<u32>,
}
