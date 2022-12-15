use crate::modules::native;

pub(crate) fn get_near_metadata() -> native::schemas::Metadata {
    native::schemas::Metadata {
        name: "NEAR blockchain native token".to_string(),
        symbol: "NEAR".to_string(),
        // TODO PHASE 2 re-check the icon. It's the best I can find
        icon: Some("https://raw.githubusercontent.com/near/near-wallet/7ef3c824404282b76b36da2dff4f3e593e7f928d/packages/frontend/src/images/near.svg".to_string()),
        decimals: 24,
    }
}
