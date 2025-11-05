use axum::{http::StatusCode, Json};
use serde::Deserialize;
use crate::db::{models::*, message_ops::*, kg_ops::*, connect::get_client};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BatchMessageIngestRequest {
    #[serde(flatten)]
    pub turns: Vec<TurnEmbedding>,
}

// ============================================================================
// Message Ingestion Handler
// ============================================================================

/// Ingest messages with their full embeddings from turn_embeddings.json
pub async fn ingest_turn_embeddings(
    Json(payload): Json<Vec<TurnEmbedding>>,
) -> Result<Json<IngestResponse>, StatusCode> {
    let start = std::time::Instant::now();
    let total_processed = payload.len();

    println!("Starting ingestion of {} turn embeddings", total_processed);

    let client = match get_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match batch_insert_messages(&client, &payload).await {
        Ok((count, errors)) => {
            println!("Successfully ingested {} messages", count);
            
            Ok(Json(IngestResponse {
                success: errors.is_empty(),
                total_processed,
                total_inserted: count,
                duration_ms: start.elapsed().as_millis(),
                errors,
            }))
        }
        Err(e) => {
            eprintln!("Error during batch insert: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Knowledge Graph Ingestion Handler
// ============================================================================

/// Ingest knowledge graph data from enhanced_pipeline_full_results.json
pub async fn ingest_knowledge_graph(
    Json(payload): Json<ConversationKnowledgeGraph>,
) -> Result<Json<IngestResponse>, StatusCode> {
    let start = std::time::Instant::now();
    
    let total_processed: usize = payload.conversations.values()
        .map(|kg| kg.nodes.len() + kg.edges.len())
        .sum();

    println!("Starting ingestion of knowledge graph with {} conversations", 
        payload.conversations.len());

    let client = match get_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match batch_insert_knowledge_graph(&client, payload).await {
        Ok((nodes, edges, errors)) => {
            println!("Successfully ingested {} nodes and {} edges", nodes, edges);
            
            Ok(Json(IngestResponse {
                success: errors.is_empty(),
                total_processed,
                total_inserted: nodes + edges,
                duration_ms: start.elapsed().as_millis(),
                errors,
            }))
        }
        Err(e) => {
            eprintln!("Error during knowledge graph insert: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Statistics Handler
// ============================================================================

/// Get statistics about ingested data
pub async fn get_statistics() -> Result<Json<serde_json::Value>, StatusCode> {
    let client = match get_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match get_kg_statistics(&client).await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            eprintln!("Error fetching statistics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

