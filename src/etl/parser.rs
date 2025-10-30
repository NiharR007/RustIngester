use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ============================================================================
// Original Models (for backward compatibility)
// ============================================================================

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

// ============================================================================
// New Models for ok.json Format
// ============================================================================

/// Node from ok.json format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

/// Edge from ok.json format
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KnowledgeEdge {
    pub source: String,
    pub relation: String,
    pub target: String,
    pub evidence_message_ids: Vec<String>,
}

/// Session graph containing nodes and edges
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

/// Complete knowledge graph data (session_id -> graph)
pub type KnowledgeGraphData = HashMap<String, SessionGraph>;

// ============================================================================
// Conversion Functions
// ============================================================================

impl KnowledgeNode {
    /// Convert to ParsedNode for existing ingestion pipeline
    pub fn to_parsed_node(&self) -> ParsedNode {
        ParsedNode {
            label: self.node_type.clone(),
            pk: self.id.clone(),
            props: Value::Null,
        }
    }
}

impl KnowledgeEdge {
    /// Convert evidence to JSON for edge properties
    pub fn to_edge_props(&self) -> Value {
        serde_json::json!({
            "evidence_message_ids": self.evidence_message_ids
        })
    }
}
