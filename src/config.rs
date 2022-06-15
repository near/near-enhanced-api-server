#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub addr: String,
    pub cors_allowed_origins: Vec<String>,
    #[serde(default)]
    pub limits: LimitsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0:3050".to_owned(),
            cors_allowed_origins: vec!["*".to_owned()],
            limits: LimitsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LimitsConfig {
    pub input_payload_max_size: usize,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            input_payload_max_size: 10 * 1024 * 1024,
        }
    }
}
