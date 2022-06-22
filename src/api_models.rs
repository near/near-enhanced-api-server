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
pub(crate) struct FtMetadataRequest {
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
pub(crate) struct FtHistoryRequest {
    pub account_id: super::types::AccountId,
    pub contract_account_id: super::types::AccountId,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct BalanceResponse {
    pub balances: Vec<CoinInfo>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct FtHistoryResponse {
    pub history: Vec<FtHistoryInfo>,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

// todo it does not work for MT
// https://nomicon.io/Standards/Tokens/MultiToken/Metadata
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct FtMetadataResponse {
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
    // todo not sure we want to add it here, but for consistency, we have to
    // it will also make the caching much harder
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct CoinInfo {
    // todo use enums here?
    // todo add metadata fields
    pub standard: String,
    pub contract_account_id: Option<super::types::AccountId>,
    pub balance: super::types::U128,
    pub symbol: String,
    pub decimals: u8,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Apiv2Schema)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub(crate) struct FtHistoryInfo {
    pub action_kind: String, // mint transfer burn
    pub affected_account_id: Option<super::types::AccountId>,
    pub delta_balance: super::types::I128,
    pub balance: super::types::U128,
    pub block_timestamp_nanos: super::types::U64,
    pub block_height: super::types::U64,
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
