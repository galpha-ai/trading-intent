//! Schema loader — reads intent schema YAML files from a directory
//! and builds a registry keyed by intent type name.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// A single intent schema loaded from YAML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSchema {
    /// Intent type name (e.g. "IMMEDIATE"). Used for matching and routing.
    pub name: String,
    /// Human-readable description of this intent type.
    pub description: String,
    /// Field definitions for validation.
    pub fields: serde_yaml::Value,
    /// Default XML template with `{{placeholder}}` slots.
    #[serde(default)]
    pub template: Option<String>,
    /// Named template variants (e.g. "buy", "sell_percentage").
    #[serde(default)]
    pub template_variants: HashMap<String, TemplateVariant>,
    /// XML shorthand expansions (e.g. `<amount>all</amount>` → percentage 100).
    #[serde(default)]
    pub xml_shorthands: Vec<XmlShorthand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariant {
    pub description: String,
    pub xml: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XmlShorthand {
    /// Pattern to match in raw XML (simple string match).
    #[serde(rename = "match")]
    pub match_pattern: String,
    /// Replacement string.
    pub expands_to: String,
    pub description: String,
}

/// Registry of all loaded intent schemas, keyed by uppercase name.
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    /// Intent schemas keyed by uppercase name. Public for test construction.
    pub schemas: HashMap<String, IntentSchema>,
}

impl SchemaRegistry {
    /// Load all `.yaml` / `.yml` files from a directory into the registry.
    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref();
        let mut schemas = HashMap::new();

        let entries = std::fs::read_dir(dir)
            .with_context(|| format!("Failed to read intent schema directory: {}", dir.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "yaml" && ext != "yml" {
                continue;
            }

            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read schema file: {}", path.display()))?;

            let schema: IntentSchema = serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse schema file: {}", path.display()))?;

            let key = schema.name.to_uppercase();
            info!(name = %schema.name, path = %path.display(), "Loaded intent schema");
            schemas.insert(key, schema);
        }

        if schemas.is_empty() {
            anyhow::bail!("No intent schemas found in {}", dir.display());
        }

        info!(count = schemas.len(), "Intent schema registry ready");
        Ok(Self { schemas })
    }

    /// Look up a schema by intent type name (case-insensitive).
    pub fn get(&self, intent_type: &str) -> Option<&IntentSchema> {
        self.schemas.get(&intent_type.to_uppercase())
    }

    /// List all registered schema names.
    pub fn list(&self) -> Vec<&IntentSchema> {
        self.schemas.values().collect()
    }

    /// Check if an intent type is known.
    pub fn contains(&self, intent_type: &str) -> bool {
        self.schemas.contains_key(&intent_type.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_schema(dir: &Path, filename: &str, content: &str) {
        let path = dir.join(filename);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn loads_schemas_from_directory() {
        let dir = TempDir::new().unwrap();
        write_schema(
            dir.path(),
            "test.yaml",
            r#"
name: TEST
description: "A test intent"
fields:
  chain_id:
    type: string
    required: true
template: "<intent><type>TEST</type></intent>"
"#,
        );

        let registry = SchemaRegistry::load_from_dir(dir.path()).unwrap();
        assert!(registry.contains("TEST"));
        assert!(registry.contains("test")); // case-insensitive
        assert!(!registry.contains("UNKNOWN"));
    }

    #[test]
    fn empty_directory_fails() {
        let dir = TempDir::new().unwrap();
        assert!(SchemaRegistry::load_from_dir(dir.path()).is_err());
    }
}
