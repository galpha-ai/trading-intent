//! Dispatcher matching — first-match-wins against config rules.

use crate::config::DispatcherConfig;
use crate::intent::ValidatedIntent;

/// Find the first dispatcher whose match rule fits the intent.
pub fn find_match<'a>(
    intent: &ValidatedIntent,
    dispatchers: &'a [DispatcherConfig],
) -> Option<&'a DispatcherConfig> {
    dispatchers.iter().find(|d| {
        let rule = &d.match_rule;

        if let Some(ref expected) = rule.intent_type {
            if !expected.eq_ignore_ascii_case(&intent.intent_type) {
                return false;
            }
        }

        if let Some(ref pattern) = rule.chain_id {
            if !glob_match::glob_match(pattern, &intent.chain_id) {
                return false;
            }
        }

        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DispatcherConfig, MatchRule};
    use chrono::Utc;
    use serde_json::json;

    fn intent(intent_type: &str, chain_id: &str) -> ValidatedIntent {
        ValidatedIntent {
            intent_id: "test".into(),
            intent_type: intent_type.into(),
            chain_id: chain_id.into(),
            payload: json!({}),
            raw_xml: String::new(),
            received_at: Utc::now(),
        }
    }

    fn dispatcher(t: Option<&str>, c: Option<&str>, ep: &str) -> DispatcherConfig {
        DispatcherConfig {
            match_rule: MatchRule {
                intent_type: t.map(Into::into),
                chain_id: c.map(Into::into),
            },
            endpoint: ep.into(),
            timeout_secs: 30,
            headers: Default::default(),
        }
    }

    #[test]
    fn exact_match() {
        let ds = vec![
            dispatcher(Some("IMMEDIATE"), Some("solana:*"), "http://sol"),
            dispatcher(Some("IMMEDIATE"), Some("eip155:*"), "http://evm"),
        ];
        assert_eq!(find_match(&intent("IMMEDIATE", "solana:mainnet-beta"), &ds).unwrap().endpoint, "http://sol");
        assert_eq!(find_match(&intent("IMMEDIATE", "eip155:1"), &ds).unwrap().endpoint, "http://evm");
    }

    #[test]
    fn no_match() {
        let ds = vec![dispatcher(Some("IMMEDIATE"), Some("solana:*"), "http://sol")];
        assert!(find_match(&intent("CONDITIONAL_ENTRY", "solana:mainnet-beta"), &ds).is_none());
    }

    #[test]
    fn wildcard_fallback() {
        let ds = vec![dispatcher(None, None, "http://fallback")];
        assert_eq!(find_match(&intent("ANYTHING", "any:chain"), &ds).unwrap().endpoint, "http://fallback");
    }

    #[test]
    fn first_wins() {
        let ds = vec![
            dispatcher(Some("IMMEDIATE"), Some("solana:*"), "http://first"),
            dispatcher(Some("IMMEDIATE"), None, "http://second"),
        ];
        assert_eq!(find_match(&intent("IMMEDIATE", "solana:devnet"), &ds).unwrap().endpoint, "http://first");
    }

    #[test]
    fn case_insensitive_intent_type() {
        let ds = vec![dispatcher(Some("IMMEDIATE"), None, "http://a")];
        assert!(find_match(&intent("immediate", "x"), &ds).is_some());
        assert!(find_match(&intent("Immediate", "x"), &ds).is_some());
    }

    #[test]
    fn complex_glob_chain_id() {
        let ds = vec![dispatcher(Some("IMMEDIATE"), Some("eip155:*"), "http://evm")];
        assert!(find_match(&intent("IMMEDIATE", "eip155:8453"), &ds).is_some());
        assert!(find_match(&intent("IMMEDIATE", "eip155:1"), &ds).is_some());
        assert!(find_match(&intent("IMMEDIATE", "solana:mainnet-beta"), &ds).is_none());
    }
}
