//! Dispatch engine — routes validated intents to configured executor endpoints.

pub mod matcher;
pub mod sender;

use crate::config::DispatcherConfig;
use crate::intent::ValidatedIntent;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatchError {
    #[error("No dispatcher matched intent_type={intent_type} chain_id={chain_id}")]
    NoMatch {
        intent_type: String,
        chain_id: String,
    },

    #[error("Executor request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Executor returned error (HTTP {status}): {body}")]
    ExecutorError { status: u16, body: String },

    #[error("Dispatch timeout after {timeout_secs}s to {endpoint}")]
    Timeout { endpoint: String, timeout_secs: u64 },
}

/// Response from the executor, forwarded to the caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResponse {
    #[serde(flatten)]
    pub executor_response: serde_json::Value,
    pub dispatched_to: String,
}

/// Dispatch a validated intent through the configured dispatchers.
pub async fn dispatch(
    intent: ValidatedIntent,
    dispatchers: &[DispatcherConfig],
    http_client: &reqwest::Client,
) -> Result<DispatchResponse, DispatchError> {
    let matched = matcher::find_match(&intent, dispatchers).ok_or_else(|| {
        DispatchError::NoMatch {
            intent_type: intent.intent_type.clone(),
            chain_id: intent.chain_id.clone(),
        }
    })?;

    sender::send(intent, matched, http_client).await
}
