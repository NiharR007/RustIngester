use axum::{
    routing::{get, post},
    Router,
    extract::DefaultBodyLimit,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::handlers;
use super::ingest_handlers;
use super::context_handlers;

pub fn create_router() -> Router {
    Router::new()
        // Health check
        .route("/status", get(handlers::health_check))
        
        // Ingestion endpoints
        .route("/ingest/session", post(handlers::ingest_session))
        .route("/ingest/batch", post(handlers::ingest_batch))
        
        // New: Message and Knowledge Graph ingestion
        .route("/ingest/messages", post(ingest_handlers::ingest_turn_embeddings))
        .route("/ingest/knowledge-graph", post(ingest_handlers::ingest_knowledge_graph))
        .route("/ingest/statistics", get(ingest_handlers::get_statistics))
        
        // Query endpoints
        .route("/query/similar", post(handlers::query_similar))
        .route("/query/session/:session_id", get(handlers::get_session))
        
        // New: LLM Context query endpoints
        .route("/query/llm-context", post(context_handlers::query_llm_context))
        .route("/query/messages", post(context_handlers::query_messages_by_ids))
        
        // Graph query endpoint
        .route("/graph/cypher", post(handlers::execute_cypher))
        
        // Middleware
        .layer(DefaultBodyLimit::max(500 * 1024 * 1024)) // 500MB limit for large ingestion
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
