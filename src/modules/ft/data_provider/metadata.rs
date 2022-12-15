use crate::modules::ft;
use crate::rpc_helpers;

// todo switch from rpc to db
pub(crate) async fn get_ft_metadata(
    rpc_client: &near_jsonrpc_client::JsonRpcClient,
    contract_id: near_primitives::types::AccountId,
    block_height: u64,
) -> crate::Result<ft::schemas::FtContractMetadata> {
    let request = rpc_helpers::get_function_call_request(
        block_height,
        contract_id.clone(),
        "ft_metadata",
        serde_json::json!({}),
    );
    let response =
        rpc_helpers::wrapped_call(rpc_client, request, block_height, &contract_id).await?;

    Ok(serde_json::from_slice::<ft::schemas::FtContractMetadata>(
        &response.result,
    )?)
}

impl From<ft::schemas::FtContractMetadata> for ft::schemas::Metadata {
    fn from(metadata: ft::schemas::FtContractMetadata) -> Self {
        ft::schemas::Metadata {
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

        let metadata = get_ft_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_ft_contract_metadata_no_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("olga.near").unwrap();

        let metadata = get_ft_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_ft_contract_metadata_other_contract_deployed() {
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str("comic.paras.near").unwrap();

        let metadata = get_ft_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }

    #[tokio::test]
    async fn test_ft_bridged_contract_metadata() {
        //https://github.com/near/near-enhanced-api-server/issues/43
        let rpc_client = init_rpc();
        let block = get_block();
        let contract = near_primitives::types::AccountId::from_str(
            "0316eb71485b0ab14103307bf65a021042c6d380.factory.bridge.near",
        )
        .unwrap();

        let metadata = get_ft_metadata(&rpc_client, contract, block.height).await;
        insta::assert_debug_snapshot!(metadata);
    }
}
