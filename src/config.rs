use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    /// Path to directory containing intent schema YAML files.
    #[serde(default = "default_schema_dir")]
    pub intent_schemas: String,
    pub dispatchers: Vec<DispatcherConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_host")]
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatcherConfig {
    #[serde(rename = "match")]
    pub match_rule: MatchRule,
    pub endpoint: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchRule {
    pub intent_type: Option<String>,
    pub chain_id: Option<String>,
}

fn default_port() -> u16 { 8080 }
fn default_host() -> String { "127.0.0.1".into() }
fn default_timeout() -> u64 { 30 }
fn default_schema_dir() -> String { "intents/".into() }

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("TIM_CONFIG_PATH")
            .unwrap_or_else(|_| "config/local.yaml".into());
        Self::from_file(&path)
    }

    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Ok(serde_yaml::from_str(&content)?)
    }
}
