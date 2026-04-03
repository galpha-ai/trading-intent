//! Schema-driven validator — checks a parsed JSON intent against its schema definition.

use super::loader::SchemaRegistry;
use regex::Regex;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Unknown intent type: {0}")]
    UnknownType(String),

    #[error("Missing required field: {path}")]
    MissingField { path: String },

    #[error("Invalid value at {path}: {reason}")]
    InvalidValue { path: String, reason: String },

    #[error("Constraint violation at {path}: {rule}")]
    ConstraintViolation { path: String, rule: String },

    #[error("Schema error: {0}")]
    SchemaError(String),
}

/// Validate a parsed intent JSON against the schema registry.
///
/// Steps:
/// 1. Extract `type` field → look up schema
/// 2. Walk `fields` definition recursively, check required/type/constraints
pub fn validate(intent: &Value, registry: &SchemaRegistry) -> Result<(), ValidationError> {
    // Extract intent type
    let intent_type = intent.get("type").and_then(|v| v.as_str()).ok_or_else(|| {
        ValidationError::MissingField {
            path: "type".into(),
        }
    })?;

    // Look up schema
    let schema = registry
        .get(intent_type)
        .ok_or_else(|| ValidationError::UnknownType(intent_type.to_string()))?;

    // Validate top-level fields
    validate_fields(intent, &schema.fields, "")?;

    Ok(())
}

/// Recursively validate fields against a schema definition.
fn validate_fields(
    value: &Value,
    fields_def: &serde_yaml::Value,
    parent_path: &str,
) -> Result<(), ValidationError> {
    let fields_map = match fields_def.as_mapping() {
        Some(m) => m,
        None => return Ok(()), // no fields to validate
    };

    for (field_name, field_def) in fields_map {
        let name = field_name.as_str().unwrap_or("");
        let path = if parent_path.is_empty() {
            name.to_string()
        } else {
            format!("{parent_path}.{name}")
        };

        let field_value = value.get(name);
        let required = field_def
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check required
        if required && (field_value.is_none() || field_value == Some(&Value::Null)) {
            return Err(ValidationError::MissingField { path });
        }

        let Some(fv) = field_value else { continue };

        // Type check
        if let Some(type_str) = field_def.get("type").and_then(|v| v.as_str()) {
            validate_type(fv, type_str, &path)?;
        }

        // Numeric constraints
        if let Some(min) = field_def.get("min").and_then(|v| v.as_f64()) {
            if let Some(n) = fv.as_f64() {
                if n < min {
                    return Err(ValidationError::InvalidValue {
                        path,
                        reason: format!("must be >= {min}"),
                    });
                }
            }
        }
        if let Some(max) = field_def.get("max").and_then(|v| v.as_f64()) {
            if let Some(n) = fv.as_f64() {
                if n > max {
                    return Err(ValidationError::InvalidValue {
                        path,
                        reason: format!("must be <= {max}"),
                    });
                }
            }
        }
        if let Some(min_ex) = field_def.get("min_exclusive").and_then(|v| v.as_f64()) {
            if let Some(n) = fv.as_f64() {
                if n <= min_ex {
                    return Err(ValidationError::InvalidValue {
                        path,
                        reason: format!("must be > {min_ex}"),
                    });
                }
            }
        }
        if let Some(max_ex) = field_def.get("max_exclusive").and_then(|v| v.as_f64()) {
            if let Some(n) = fv.as_f64() {
                if n >= max_ex {
                    return Err(ValidationError::InvalidValue {
                        path,
                        reason: format!("must be < {max_ex}"),
                    });
                }
            }
        }

        // Pattern constraint (regex)
        if let Some(pattern_str) = field_def.get("pattern").and_then(|v| v.as_str()) {
            if let Some(s) = fv.as_str() {
                let re = Regex::new(pattern_str).map_err(|e| {
                    ValidationError::SchemaError(format!(
                        "invalid regex pattern '{pattern_str}': {e}"
                    ))
                })?;
                if !re.is_match(s) {
                    return Err(ValidationError::InvalidValue {
                        path,
                        reason: format!("does not match pattern: {pattern_str}"),
                    });
                }
            }
        }

        // Enum constraint
        if let Some(enum_values) = field_def.get("enum").and_then(|v| v.as_sequence()) {
            if let Some(s) = fv.as_str() {
                let allowed: Vec<&str> = enum_values.iter().filter_map(|v| v.as_str()).collect();
                if !allowed.iter().any(|a| a.eq_ignore_ascii_case(s)) {
                    return Err(ValidationError::InvalidValue {
                        path,
                        reason: format!("must be one of: {allowed:?}"),
                    });
                }
            }
        }

        // Recurse into nested object fields
        if let Some(nested_fields) = field_def.get("fields") {
            if fv.is_object() {
                validate_fields(fv, nested_fields, &path)?;
            }
        }

        // Array items validation
        if let Some(items_def) = field_def.get("items") {
            if let Some(arr) = fv.as_array() {
                for (i, element) in arr.iter().enumerate() {
                    let elem_path = format!("{path}[{i}]");
                    // Type check on each element
                    if let Some(item_type) = items_def.get("type").and_then(|v| v.as_str()) {
                        validate_type(element, item_type, &elem_path)?;
                    }
                    // Recurse into element fields if defined
                    if let Some(item_fields) = items_def.get("fields") {
                        if element.is_object() {
                            validate_fields(element, item_fields, &elem_path)?;
                        }
                    }
                }
            }
        }

        // one_of constraint (for action variants like buy/sell)
        if let Some(one_of) = field_def.get("one_of").and_then(|v| v.as_sequence()) {
            validate_one_of(fv, one_of, &path)?;
        }

        // constraints list (e.g. exactly_one_of)
        if let Some(constraints) = field_def.get("constraints").and_then(|v| v.as_sequence()) {
            for constraint in constraints {
                if let Some(exactly_one) = constraint
                    .get("exactly_one_of")
                    .and_then(|v| v.as_sequence())
                {
                    let field_names: Vec<&str> =
                        exactly_one.iter().filter_map(|v| v.as_str()).collect();
                    let present_count = field_names
                        .iter()
                        .filter(|f| fv.get(**f).is_some() && fv.get(**f) != Some(&Value::Null))
                        .count();
                    if present_count != 1 {
                        return Err(ValidationError::ConstraintViolation {
                            path: path.clone(),
                            rule: format!("exactly one of {field_names:?} must be present"),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn validate_one_of(
    value: &Value,
    variants: &[serde_yaml::Value],
    path: &str,
) -> Result<(), ValidationError> {
    // Each variant is a mapping with a single key (the variant name)
    // The value at `path` must contain exactly one of these keys
    let variant_names: Vec<String> = variants
        .iter()
        .filter_map(|v| {
            v.as_mapping()
                .and_then(|m| m.keys().next())
                .and_then(|k| k.as_str())
                .map(String::from)
        })
        .collect();

    let present: Vec<&String> = variant_names
        .iter()
        .filter(|name| value.get(name.as_str()).is_some())
        .collect();

    if present.is_empty() {
        return Err(ValidationError::ConstraintViolation {
            path: path.into(),
            rule: format!("must contain one of: {variant_names:?}"),
        });
    }
    if present.len() > 1 {
        return Err(ValidationError::ConstraintViolation {
            path: path.into(),
            rule: format!("must contain only one of: {variant_names:?}, found: {present:?}"),
        });
    }

    // Validate the chosen variant's fields
    let chosen_name = present[0];
    let chosen_value = &value[chosen_name.as_str()];
    let variant_def = variants
        .iter()
        .find_map(|v| {
            v.as_mapping()?
                .get(serde_yaml::Value::String(chosen_name.clone()))
        })
        .and_then(|v| v.get("fields"));

    if let Some(fields) = variant_def {
        let variant_path = format!("{path}.{chosen_name}");
        validate_fields(chosen_value, fields, &variant_path)?;
    }

    Ok(())
}

fn validate_type(value: &Value, expected: &str, path: &str) -> Result<(), ValidationError> {
    let ok = match expected {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        _ => true, // unknown type — skip check
    };
    if !ok {
        let actual = value_type_name(value);
        return Err(ValidationError::InvalidValue {
            path: path.into(),
            reason: format!("expected {expected}, got {actual}"),
        });
    }
    Ok(())
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::loader::IntentSchema;
    use serde_json::json;

    fn test_registry() -> SchemaRegistry {
        let yaml = r#"
name: TEST
description: test
fields:
  chain_id:
    type: string
    required: true
  amount:
    type: number
    min_exclusive: 0
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("TEST".to_string(), schema);
        SchemaRegistry { schemas }
    }

    #[test]
    fn valid_intent() {
        let registry = test_registry();
        let intent = json!({"type": "TEST", "chain_id": "solana:mainnet-beta", "amount": 1.0});
        assert!(validate(&intent, &registry).is_ok());
    }

    #[test]
    fn missing_required_field() {
        let registry = test_registry();
        let intent = json!({"type": "TEST"});
        let err = validate(&intent, &registry).unwrap_err();
        assert!(matches!(err, ValidationError::MissingField { .. }));
    }

    #[test]
    fn invalid_amount() {
        let registry = test_registry();
        let intent = json!({"type": "TEST", "chain_id": "x", "amount": -1.0});
        let err = validate(&intent, &registry).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidValue { .. }));
    }

    #[test]
    fn unknown_type() {
        let registry = test_registry();
        let intent = json!({"type": "NOPE", "chain_id": "x"});
        let err = validate(&intent, &registry).unwrap_err();
        assert!(matches!(err, ValidationError::UnknownType(_)));
    }

    #[test]
    fn max_constraint() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  pct: { type: number, max: 100 }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        let intent = json!({"type": "T", "chain_id": "x", "pct": 101.0});
        assert!(matches!(
            validate(&intent, &registry).unwrap_err(),
            ValidationError::InvalidValue { .. }
        ));

        let intent = json!({"type": "T", "chain_id": "x", "pct": 100.0});
        assert!(validate(&intent, &registry).is_ok());
    }

    #[test]
    fn max_exclusive_boundary() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  val: { type: number, max_exclusive: 10 }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        // Exactly 10 should fail (exclusive)
        let intent = json!({"type": "T", "chain_id": "x", "val": 10.0});
        assert!(matches!(
            validate(&intent, &registry).unwrap_err(),
            ValidationError::InvalidValue { .. }
        ));

        // 9.99 should pass
        let intent = json!({"type": "T", "chain_id": "x", "val": 9.99});
        assert!(validate(&intent, &registry).is_ok());
    }

    #[test]
    fn pattern_match_success() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  addr: { type: string, pattern: "^0x[0-9a-fA-F]{40}$" }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        let intent = json!({"type": "T", "chain_id": "x", "addr": "0x1234567890abcdef1234567890abcdef12345678"});
        assert!(validate(&intent, &registry).is_ok());
    }

    #[test]
    fn pattern_match_failure() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  addr: { type: string, pattern: "^0x[0-9a-fA-F]{40}$" }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        let intent = json!({"type": "T", "chain_id": "x", "addr": "not_an_address"});
        let err = validate(&intent, &registry).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidValue { .. }));
    }

    #[test]
    fn enum_multiple_values() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  side: { type: string, enum: [buy, sell, hold] }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        let intent = json!({"type": "T", "chain_id": "x", "side": "sell"});
        assert!(validate(&intent, &registry).is_ok());

        let intent = json!({"type": "T", "chain_id": "x", "side": "invalid"});
        assert!(matches!(
            validate(&intent, &registry).unwrap_err(),
            ValidationError::InvalidValue { .. }
        ));
    }

    fn one_of_registry() -> SchemaRegistry {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  action:
    type: object
    required: true
    one_of:
      - buy:
          fields:
            amount: { type: number, required: true }
      - sell:
          fields:
            amount: { type: number, required: true }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        SchemaRegistry { schemas }
    }

    #[test]
    fn one_of_valid() {
        let registry = one_of_registry();
        let intent = json!({"type": "T", "chain_id": "x", "action": {"buy": {"amount": 1.0}}});
        assert!(validate(&intent, &registry).is_ok());
    }

    #[test]
    fn one_of_none_present() {
        let registry = one_of_registry();
        let intent = json!({"type": "T", "chain_id": "x", "action": {"other": true}});
        let err = validate(&intent, &registry).unwrap_err();
        assert!(matches!(err, ValidationError::ConstraintViolation { .. }));
    }

    #[test]
    fn one_of_both_present() {
        let registry = one_of_registry();
        let intent = json!({"type": "T", "chain_id": "x", "action": {"buy": {"amount": 1.0}, "sell": {"amount": 2.0}}});
        let err = validate(&intent, &registry).unwrap_err();
        assert!(matches!(err, ValidationError::ConstraintViolation { .. }));
    }

    #[test]
    fn exactly_one_of_valid() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  opts:
    type: object
    required: true
    fields:
      a: { type: number }
      b: { type: number }
    constraints:
      - exactly_one_of: [a, b]
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        let intent = json!({"type": "T", "chain_id": "x", "opts": {"a": 1.0}});
        assert!(validate(&intent, &registry).is_ok());
    }

    #[test]
    fn exactly_one_of_both() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  opts:
    type: object
    required: true
    fields:
      a: { type: number }
      b: { type: number }
    constraints:
      - exactly_one_of: [a, b]
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        let intent = json!({"type": "T", "chain_id": "x", "opts": {"a": 1.0, "b": 2.0}});
        let err = validate(&intent, &registry).unwrap_err();
        assert!(matches!(err, ValidationError::ConstraintViolation { .. }));
    }

    #[test]
    fn array_items_validation() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  items:
    type: array
    items:
      type: object
      fields:
        amount: { type: number, required: true }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        // Valid array
        let intent =
            json!({"type": "T", "chain_id": "x", "items": [{"amount": 1.0}, {"amount": 2.0}]});
        assert!(validate(&intent, &registry).is_ok());

        // Missing required field in array element
        let intent = json!({"type": "T", "chain_id": "x", "items": [{"amount": 1.0}, {}]});
        assert!(matches!(
            validate(&intent, &registry).unwrap_err(),
            ValidationError::MissingField { .. }
        ));
    }

    #[test]
    fn nested_object_deep() {
        let yaml = r#"
name: T
description: t
fields:
  chain_id: { type: string, required: true }
  level1:
    type: object
    required: true
    fields:
      level2:
        type: object
        required: true
        fields:
          value: { type: number, required: true, min: 0 }
"#;
        let schema: IntentSchema = serde_yaml::from_str(yaml).unwrap();
        let mut schemas = std::collections::HashMap::new();
        schemas.insert("T".to_string(), schema);
        let registry = SchemaRegistry { schemas };

        let intent = json!({"type": "T", "chain_id": "x", "level1": {"level2": {"value": 5.0}}});
        assert!(validate(&intent, &registry).is_ok());

        let intent = json!({"type": "T", "chain_id": "x", "level1": {"level2": {"value": -1.0}}});
        assert!(matches!(
            validate(&intent, &registry).unwrap_err(),
            ValidationError::InvalidValue { .. }
        ));
    }
}
