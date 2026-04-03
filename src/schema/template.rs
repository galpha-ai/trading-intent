//! Template service — exposes intent schemas as agent-facing templates.

use super::loader::{IntentSchema, SchemaRegistry};
use serde::Serialize;

/// Summary of one intent type, returned by `GET /api/v1/templates`.
#[derive(Debug, Clone, Serialize)]
pub struct TemplateSummary {
    pub name: String,
    pub description: String,
    pub variants: Vec<String>,
}

/// Full template info for one intent type, returned by `GET /api/v1/templates/{type}`.
#[derive(Debug, Clone, Serialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    /// Default XML template with `{{placeholder}}` slots.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// Named variants with description and XML.
    pub variants: Vec<VariantInfo>,
    /// Field descriptions for agent reference.
    pub fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct VariantInfo {
    pub name: String,
    pub description: String,
    pub xml: String,
}

/// List all available intent schemas as summaries.
pub fn list_templates(registry: &SchemaRegistry) -> Vec<TemplateSummary> {
    let mut result: Vec<TemplateSummary> = registry
        .list()
        .iter()
        .map(|s| TemplateSummary {
            name: s.name.clone(),
            description: s.description.clone(),
            variants: s.template_variants.keys().cloned().collect(),
        })
        .collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// Get full template info for one intent type.
pub fn get_template(schema: &IntentSchema) -> TemplateInfo {
    let variants = schema
        .template_variants
        .iter()
        .map(|(name, v)| VariantInfo {
            name: name.clone(),
            description: v.description.clone(),
            xml: v.xml.clone(),
        })
        .collect();

    // Convert fields from serde_yaml::Value to serde_json::Value for the response
    let fields = serde_json::to_value(&schema.fields).unwrap_or(serde_json::Value::Null);

    TemplateInfo {
        name: schema.name.clone(),
        description: schema.description.clone(),
        template: schema.template.clone(),
        variants,
        fields,
    }
}
