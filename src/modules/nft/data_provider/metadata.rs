use crate::modules::nft;
use crate::{errors, rpc_helpers, types};
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub(crate) async fn get_nft_contract_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    block_height: u64,
) -> crate::Result<nft::schemas::NftContractMetadata> {
    let request = rpc_helpers::get_function_call_request(
        block_height,
        contract_id.clone(),
        "nft_metadata",
        serde_json::json!({}),
    );
    let response = match rpc_helpers::wrapped_call(rpc_client, request, block_height, &contract_id)
        .await
    {
        Ok(response) => response,
        Err(err) => {
            println!("{}", err.message);
            if err
                .message
                .contains("called `Option::unwrap()` on a `None` value")
            {
                return Err(errors::ErrorKind::ContractError(
                    "The contract did not provide NFT Metadata which is a required part of NFT NEP 171".to_string(),
                )
                    .into());
            }
            return Err(err);
        }
    };

    nft::schemas::NftContractMetadata::try_from(serde_json::from_slice::<NFTContractMetadata>(
        &response.result,
    )?)
}

// Metadata is the required part of the standard.
// Unfortunately, some contracts (e.g. `nft.nearapps.near`) do not implement it.
// We should give at least anything for such contracts when we serve the overview information.
pub(crate) fn get_default_nft_contract_metadata() -> nft::schemas::NftContractMetadata {
    nft::schemas::NftContractMetadata {
        spec: "nft-1.0.0".to_string(),
        name: "The contract did not provide the metadata".to_string(),
        symbol: "The contract did not provide the symbol".to_string(),
        icon: None,
        base_uri: None,
        reference: None,
        reference_hash: None,
    }
}

// Taken from https://github.com/near/near-sdk-rs/blob/master/near-contract-standards/src/non_fungible_token/metadata.rs
/// Metadata for the NFT contract itself.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NFTContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Mosaics"
    pub symbol: String,            // required, ex. "MOSIAC"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>, // Centralized gateway known to have reliable access to decentralized data_provider assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<types::vector::Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

impl TryFrom<NFTContractMetadata> for nft::schemas::NftContractMetadata {
    type Error = errors::Error;

    fn try_from(metadata: NFTContractMetadata) -> crate::Result<Self> {
        Ok(Self {
            spec: metadata.spec,
            name: metadata.name,
            symbol: metadata.symbol,
            icon: metadata.icon,
            base_uri: metadata.base_uri,
            reference: metadata.reference,
            reference_hash: types::vector::base64_to_string(&metadata.reference_hash)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::tests::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_nft_contract_metadata() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_contract_metadata_no_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_contract_metadata_other_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("usn").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_nft_contract_metadata_broken_contract() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("nft.nearapps.near").unwrap();

        let metadata = get_nft_contract_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }
}
