use crate::modules::nft;
use crate::{errors, rpc_helpers};

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

    Ok(serde_json::from_slice::<nft::schemas::NftContractMetadata>(
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
