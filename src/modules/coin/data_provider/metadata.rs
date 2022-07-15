use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::modules::coin;
use crate::{rpc_helpers, types};

pub(crate) async fn get_ft_contract_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    block_height: u64,
) -> crate::Result<coin::schemas::FtContractMetadata> {
    let request = rpc_helpers::get_function_call_request(
        block_height,
        contract_id.clone(),
        "ft_metadata",
        serde_json::json!({}),
    );
    let response =
        rpc_helpers::wrapped_call(rpc_client, request, block_height, &contract_id).await?;

    let metadata = serde_json::from_slice::<FtMetadata>(&response.result)?;
    Ok(coin::schemas::FtContractMetadata {
        spec: metadata.spec,
        name: metadata.name,
        symbol: metadata.symbol,
        icon: metadata.icon,
        decimals: metadata.decimals,
        reference: metadata.reference,
        reference_hash: types::vector::base64_to_string(&metadata.reference_hash)?,
    })
}

pub(crate) fn get_near_metadata() -> coin::schemas::Metadata {
    coin::schemas::Metadata {
        name: "NEAR blockchain native token".to_string(),
        symbol: "NEAR".to_string(),
        // TODO PHASE 2 re-check the icon. It's the best I can find
        icon: Some("https://raw.githubusercontent.com/near/near-wallet/7ef3c824404282b76b36da2dff4f3e593e7f928d/packages/frontend/src/images/near.svg".to_string()),
        decimals: 24,
    }
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/fungible_token/metadata.rs
#[derive(BorshDeserialize, BorshSerialize, Clone, Deserialize, Serialize)]
pub struct FtMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<types::vector::Base64VecU8>,
    pub decimals: u8,
}

impl From<coin::schemas::FtContractMetadata> for coin::schemas::Metadata {
    fn from(metadata: coin::schemas::FtContractMetadata) -> Self {
        coin::schemas::Metadata {
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            decimals: metadata.decimals,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_ft_contract_metadata() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();

        let metadata = get_ft_contract_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_ft_contract_metadata_no_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();

        let metadata = get_ft_contract_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_ft_contract_metadata_other_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();

        let metadata = get_ft_contract_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }
}
