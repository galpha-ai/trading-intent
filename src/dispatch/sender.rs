//! HTTP sender — delivers the dispatch payload to the executor.

use super::{DispatchError, DispatchResponse};
use crate::config::DispatcherConfig;
use crate::intent::ValidatedIntent;
use std::time::Duration;
use tracing::{error, info};

/// Send a validated intent to the executor endpoint.
pub async fn send(
    intent: ValidatedIntent,
    dispatcher: &DispatcherConfig,
    client: &reqwest::Client,
) -> Result<DispatchResponse, DispatchError> {
    let endpoint = &dispatcher.endpoint;

    info!(
        intent_id = %intent.intent_id,
        intent_type = %intent.intent_type,
        chain_id = %intent.chain_id,
        endpoint = %endpoint,
        "Dispatching intent"
    );

    let mut req = client
        .post(endpoint)
        .timeout(Duration::from_secs(dispatcher.timeout_secs))
        .header("X-TIM-Intent-ID", &intent.intent_id)
        .header("X-TIM-Intent-Type", &intent.intent_type)
        .header("X-TIM-Chain-ID", &intent.chain_id);

    for (k, v) in &dispatcher.headers {
        req = req.header(k, v);
    }

    let resp = req.json(&intent).send().await.map_err(|e| {
        if e.is_timeout() {
            DispatchError::Timeout {
                endpoint: endpoint.clone(),
                timeout_secs: dispatcher.timeout_secs,
            }
        } else {
            DispatchError::RequestFailed(e)
        }
    })?;

    let status = resp.status().as_u16();
    if status >= 400 {
        let body = resp.text().await.unwrap_or_default();
        error!(status, endpoint = %endpoint, "Executor error");
        return Err(DispatchError::ExecutorError { status, body });
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| {
        DispatchError::ExecutorError {
            status,
            body: format!("Bad executor response: {}", e),
        }
    })?;

    Ok(DispatchResponse {
        executor_response: body,
        dispatched_to: endpoint.clone(),
    })
}
