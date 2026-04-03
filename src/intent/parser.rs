//! Generic recursive XML parser.
//!
//! Converts any `<intent>` XML into a serde_json::Value tree.
//! The parser is:
//! - Case-insensitive (element names lowercased)
//! - Whitespace-tolerant (text content trimmed)
//! - Forgiving (unknown elements skipped)
//! - Shorthand-aware (expands configured shorthands before parsing)

use anyhow::{anyhow, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde_json::{json, Value};

use crate::schema::loader::XmlShorthand;

/// Maximum allowed XML payload size in bytes (1 MB).
const MAX_XML_SIZE: usize = 1_048_576;

/// Maximum allowed XML nesting depth.
const MAX_XML_DEPTH: usize = 64;

/// Parse raw XML into a JSON Value, applying shorthands.
pub fn parse_xml(xml: &str, shorthands: &[XmlShorthand]) -> Result<Value> {
    if xml.len() > MAX_XML_SIZE {
        return Err(anyhow!(
            "XML payload exceeds maximum size of {} bytes",
            MAX_XML_SIZE
        ));
    }

    // Apply shorthands to raw XML
    let mut processed = xml.trim().to_string();
    for shorthand in shorthands {
        processed = processed.replace(&shorthand.match_pattern, &shorthand.expands_to);
    }

    let mut reader = Reader::from_str(&processed);
    reader.config_mut().trim_text(true);

    // Find the <intent> root element
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())?.to_lowercase();
                if name == "intent" {
                    return parse_element_children(&mut reader, "intent", 0);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parsing error: {}", e)),
            _ => {}
        }
    }

    Err(anyhow!("No <intent> element found in XML"))
}

/// Recursively parse children of an element into a JSON object.
///
/// Strategy:
/// - Child elements become object keys
/// - Text-only content becomes a JSON string or number
/// - Nested elements recurse
/// - Duplicate keys at the same level are collected into arrays
fn parse_element_children(reader: &mut Reader<&[u8]>, parent_name: &str, depth: usize) -> Result<Value> {
    if depth > MAX_XML_DEPTH {
        return Err(anyhow!(
            "XML nesting exceeds maximum depth of {}",
            MAX_XML_DEPTH
        ));
    }

    let mut obj = serde_json::Map::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())?.to_lowercase();

                // Check if this child has sub-elements or is text-only
                let child_value = parse_child(reader, &name, depth + 1)?;

                // Handle duplicate keys (collect into array)
                if let Some(existing) = obj.get_mut(&name) {
                    match existing {
                        Value::Array(arr) => arr.push(child_value),
                        _ => {
                            let prev = existing.clone();
                            *existing = json!([prev, child_value]);
                        }
                    }
                } else {
                    obj.insert(name, child_value);
                }
            }
            Ok(Event::End(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())?.to_lowercase();
                if name == parent_name {
                    break;
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.trim().to_string();
                if !text.is_empty() && obj.is_empty() {
                    // Text-only element — return the text directly
                    return Ok(coerce_value(&text));
                }
            }
            Ok(Event::Empty(e)) => {
                // Self-closing tag like <immediate/>
                let name = std::str::from_utf8(e.name().as_ref())?.to_lowercase();
                obj.insert(name, json!(true));
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parsing error: {}", e)),
            _ => {}
        }
    }

    Ok(Value::Object(obj))
}

/// Parse a single child element — could be text-only or nested.
fn parse_child(reader: &mut Reader<&[u8]>, element_name: &str, depth: usize) -> Result<Value> {
    if depth > MAX_XML_DEPTH {
        return Err(anyhow!(
            "XML nesting exceeds maximum depth of {}",
            MAX_XML_DEPTH
        ));
    }

    let mut has_children = false;
    let mut text_content = String::new();
    let mut obj = serde_json::Map::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                has_children = true;
                let name = std::str::from_utf8(e.name().as_ref())?.to_lowercase();
                let child_value = parse_child(reader, &name, depth + 1)?;

                if let Some(existing) = obj.get_mut(&name) {
                    match existing {
                        Value::Array(arr) => arr.push(child_value),
                        _ => {
                            let prev = existing.clone();
                            *existing = json!([prev, child_value]);
                        }
                    }
                } else {
                    obj.insert(name, child_value);
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape()?.trim().to_string();
                if !text.is_empty() {
                    text_content = text;
                }
            }
            Ok(Event::Empty(e)) => {
                has_children = true;
                let name = std::str::from_utf8(e.name().as_ref())?.to_lowercase();
                obj.insert(name, json!(true));
            }
            Ok(Event::End(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())?.to_lowercase();
                if name == element_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parsing error: {}", e)),
            _ => {}
        }
    }

    if has_children {
        Ok(Value::Object(obj))
    } else {
        Ok(coerce_value(&text_content))
    }
}

/// Coerce a text string into the most appropriate JSON value type.
fn coerce_value(s: &str) -> Value {
    if s.is_empty() {
        return Value::Null;
    }
    // Boolean
    match s.to_lowercase().as_str() {
        "true" => return json!(true),
        "false" => return json!(false),
        _ => {}
    }
    // Number
    if let Ok(n) = s.parse::<f64>() {
        // Preserve integers
        if n.fract() == 0.0 && !s.contains('.') {
            if let Ok(i) = s.parse::<i64>() {
                return json!(i);
            }
        }
        return json!(n);
    }
    // String
    json!(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_immediate_buy() {
        let xml = r#"
        <intent>
          <type>IMMEDIATE</type>
          <chain_id>solana:mainnet-beta</chain_id>
          <entry>
            <condition><immediate>true</immediate></condition>
            <action>
              <buy>
                <amount>0.1</amount>
                <quote>SOL_ADDR</quote>
                <base>USDC_ADDR</base>
              </buy>
            </action>
          </entry>
        </intent>"#;

        let result = parse_xml(xml, &[]).unwrap();
        assert_eq!(result["type"], "IMMEDIATE");
        assert_eq!(result["chain_id"], "solana:mainnet-beta");
        assert_eq!(result["entry"]["action"]["buy"]["amount"], 0.1);
        assert_eq!(result["entry"]["action"]["buy"]["quote"], "SOL_ADDR");
    }

    #[test]
    fn parse_sell_percentage() {
        let xml = r#"
        <intent>
          <type>IMMEDIATE</type>
          <chain_id>solana:mainnet-beta</chain_id>
          <entry>
            <condition><immediate>true</immediate></condition>
            <action>
              <sell>
                <relative><percentage>50.0</percentage></relative>
                <quote>SOL</quote>
                <base>USDC</base>
              </sell>
            </action>
          </entry>
        </intent>"#;

        let result = parse_xml(xml, &[]).unwrap();
        assert_eq!(result["entry"]["action"]["sell"]["relative"]["percentage"], 50.0);
    }

    #[test]
    fn shorthand_expansion() {
        let shorthands = vec![XmlShorthand {
            match_pattern: "<amount>all</amount>".into(),
            expands_to: "<relative><percentage>100.0</percentage></relative>".into(),
            description: "sell all".into(),
        }];

        let xml = r#"
        <intent>
          <type>IMMEDIATE</type>
          <chain_id>solana:mainnet-beta</chain_id>
          <entry>
            <condition><immediate>true</immediate></condition>
            <action>
              <sell>
                <amount>all</amount>
                <quote>SOL</quote>
                <base>USDC</base>
              </sell>
            </action>
          </entry>
        </intent>"#;

        let result = parse_xml(xml, &shorthands).unwrap();
        assert_eq!(result["entry"]["action"]["sell"]["relative"]["percentage"], 100.0);
    }

    #[test]
    fn case_insensitive() {
        let xml = r#"<Intent><Type>IMMEDIATE</Type><Chain_Id>x</Chain_Id></Intent>"#;
        let result = parse_xml(xml, &[]).unwrap();
        assert_eq!(result["type"], "IMMEDIATE");
        assert_eq!(result["chain_id"], "x");
    }

    #[test]
    fn unknown_elements_skipped() {
        let xml = r#"
        <intent>
          <type>IMMEDIATE</type>
          <chain_id>x</chain_id>
          <unknown_field>ignored</unknown_field>
        </intent>"#;

        let result = parse_xml(xml, &[]).unwrap();
        assert_eq!(result["type"], "IMMEDIATE");
        // unknown_field is present but doesn't cause error
        assert_eq!(result["unknown_field"], "ignored");
    }

    #[test]
    fn missing_intent_root() {
        let xml = "<foo><bar>baz</bar></foo>";
        assert!(parse_xml(xml, &[]).is_err());
    }

    #[test]
    fn empty_xml_string() {
        assert!(parse_xml("", &[]).is_err());
        assert!(parse_xml("   ", &[]).is_err());
    }

    #[test]
    fn xml_size_limit_exceeded() {
        // Create XML larger than 1MB
        let payload = "x".repeat(MAX_XML_SIZE + 1);
        let xml = format!("<intent><type>TEST</type><data>{}</data></intent>", payload);
        let err = parse_xml(&xml, &[]).unwrap_err();
        assert!(err.to_string().contains("maximum size"));
    }

    #[test]
    fn xml_depth_limit_exceeded() {
        // Build XML with depth > MAX_XML_DEPTH
        let mut xml = String::new();
        xml.push_str("<intent>");
        for i in 0..MAX_XML_DEPTH + 2 {
            xml.push_str(&format!("<e{}>", i));
        }
        xml.push_str("value");
        for i in (0..MAX_XML_DEPTH + 2).rev() {
            xml.push_str(&format!("</e{}>", i));
        }
        xml.push_str("</intent>");
        let err = parse_xml(&xml, &[]).unwrap_err();
        assert!(err.to_string().contains("maximum depth"));
    }

    #[test]
    fn self_closing_in_nested() {
        let xml = r#"
        <intent>
          <type>TEST</type>
          <entry>
            <flag/>
            <other>value</other>
          </entry>
        </intent>"#;
        let result = parse_xml(xml, &[]).unwrap();
        assert_eq!(result["entry"]["flag"], true);
        assert_eq!(result["entry"]["other"], "value");
    }
}
