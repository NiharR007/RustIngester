use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedNode {
    pub label: String,
    pub pk: String,
    #[serde(default)]
    pub props: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedTriplet {
    pub id: i64,
    pub subject: ParsedNode,
    pub relationship: String,
    pub object: ParsedNode,
    #[serde(default)]
    pub edge_props: serde_json::Value,
}

impl From<&str> for ParsedNode {
    fn from(s: &str) -> Self {
        Self {
            label: "Node".to_string(),
            pk: s.to_string(),
            props: Value::Null,
        }
    }
}

impl Default for ParsedNode {
    fn default() -> Self {
        Self {
            label: String::new(),
            pk: String::new(),
            props: Value::Null,
        }
    }
}

impl Default for ParsedTriplet {
    fn default() -> Self {
        Self {
            id: 0,
            subject: ParsedNode::default(),
            relationship: String::new(),
            object: ParsedNode::default(),
            edge_props: Value::Null,
        }
    }
}

/// Dummy parser: assume JSON line already matches ParsedTriplet.
/// Replace with real extractor for NRN strings/artifacts.
pub fn parse_line(line: &str) -> anyhow::Result<ParsedTriplet> {
    let t: ParsedTriplet = serde_json::from_str(line)?;
    Ok(t)
}
