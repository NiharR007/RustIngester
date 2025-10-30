use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::handlers;

pub fn create_router() -> Router {
    Router::new()
        // Health check
        .route("/status", get(handlers::health_check))
        
        // Ingestion endpoints
        .route("/ingest/session", post(handlers::ingest_session))
        .route("/ingest/batch", post(handlers::ingest_batch))
        
        // Query endpoints
        .route("/query/similar", post(handlers::query_similar))
        .route("/query/session/:session_id", get(handlers::get_session))
        
        // Graph query endpoint
        .route("/graph/cypher", post(handlers::execute_cypher))
        
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
