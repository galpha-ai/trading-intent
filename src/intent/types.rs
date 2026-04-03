//! ValidatedIntent — thin metadata wrapper around schema-validated JSON.
//!
//! TIM does not define Rust structs for each intent type.
//! The schema YAML is the source of truth; the payload is serde_json::Value.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A parsed, validated intent ready for dispatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatedIntent {
    /// Unique ID assigned by TIM at parse time.
    pub intent_id: String,
    /// Intent type string from the XML (e.g. "IMMEDIATE").
    pub intent_type: String,
    /// Target chain in CAIP-2 format.
    pub chain_id: String,
    /// The full validated JSON payload (everything inside `<intent>`).
    pub payload: serde_json::Value,
    /// Raw XML source.
    pub raw_xml: String,
    /// Timestamp when TIM received this intent.
    pub received_at: DateTime<Utc>,
}
