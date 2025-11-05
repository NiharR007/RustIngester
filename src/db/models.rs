use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// Turn Embeddings Models (from turn_embeddings.json)
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TurnEmbedding {
    pub message_id: Uuid,
    pub conversation_id: Uuid,
    pub actual_text: String,
    pub embedding: Vec<f32>, // 768-dim Nomic embeddings
}

// ============================================================================
// Knowledge Graph Models (from enhanced_pipeline_full_results.json)
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
pub struct ConversationKnowledgeGraph {
    #[serde(flatten)]
    pub conversations: HashMap<Uuid, KnowledgeGraphData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KnowledgeGraphData {
    pub nodes: Vec<KGNode>,
    pub edges: Vec<KGEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KGNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KGEdge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub evidence_message_ids: Vec<Uuid>,
}

// ============================================================================
// Database Entity Models
// ============================================================================

#[derive(Debug, Serialize, Clone)]
pub struct Conversation {
    pub conversation_id: Uuid,
}

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub message_id: Uuid,
    pub conversation_id: Uuid,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct MessageWithRelevance {
    pub message_id: Uuid,
    pub conversation_id: Uuid,
    pub content: String,
    pub relevance_score: f32,
}

// ============================================================================
// LLM Context Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct LLMContextMessage {
    pub role: String,
    pub content: String,
    pub message_id: Uuid,
    pub relevance_score: f32,
}

#[derive(Debug, Serialize)]
pub struct FormattedLLMContext {
    pub messages: Vec<LLMContextMessage>,
    pub total_tokens_estimate: usize,
    pub context_window_used: f32, // percentage
    pub unique_conversations: usize,
}

// ============================================================================
// API Response Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub success: bool,
    pub total_processed: usize,
    pub total_inserted: usize,
    pub duration_ms: u128,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct KGEdgeWithContext {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub evidence_message_ids: Vec<Uuid>,
    pub conversation_id: Uuid,
}

