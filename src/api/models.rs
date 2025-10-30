use serde::{Deserialize, Serialize};
use crate::etl::parser::{SessionGraph, KnowledgeGraphData};
use crate::ingest::{SessionIngestStats, BatchIngestStats};

// ============================================================================
// Request Models
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct IngestSessionRequest {
    pub session_id: String,
    pub graph: SessionGraph,
}

#[derive(Debug, Deserialize)]
pub struct IngestBatchRequest {
    pub sessions: KnowledgeGraphData,
}

#[derive(Debug, Deserialize)]
pub struct QuerySimilarRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: i64,
    #[serde(default)]
    pub threshold: Option<f32>,
}

fn default_top_k() -> i64 {
    5
}

#[derive(Debug, Deserialize)]
pub struct CypherQueryRequest {
    pub query: String,
}

// ============================================================================
// Response Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub database: String,
    pub age_extension: String,
    pub graph_name: String,
    pub total_sessions: i64,
    pub total_nodes: i64,
    pub total_edges: i64,
}

#[derive(Debug, Serialize)]
pub struct IngestSessionResponse {
    pub session_id: String,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub embeddings_created: usize,
    pub duration_ms: u64,
}

impl From<SessionIngestStats> for IngestSessionResponse {
    fn from(stats: SessionIngestStats) -> Self {
        Self {
            session_id: stats.session_id,
            nodes_created: stats.nodes_created,
            edges_created: stats.edges_created,
            embeddings_created: stats.embeddings_created,
            duration_ms: stats.duration_ms,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IngestBatchResponse {
    pub total_sessions: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_embeddings: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

impl From<BatchIngestStats> for IngestBatchResponse {
    fn from(stats: BatchIngestStats) -> Self {
        Self {
            total_sessions: stats.total_sessions,
            total_nodes: stats.total_nodes,
            total_edges: stats.total_edges,
            total_embeddings: stats.total_embeddings,
            duration_ms: stats.duration_ms,
            errors: stats.errors,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EdgeResult {
    pub source: String,
    pub relation: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct SimilarityResult {
    pub session_id: String,
    pub edge: EdgeResult,
    pub similarity: f32,
    pub distance: f32,
    pub evidence_message_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct QuerySimilarResponse {
    pub results: Vec<SimilarityResult>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct SessionGraphResponse {
    pub session_id: String,
    pub graph: SessionGraph,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
        }
    }
}
