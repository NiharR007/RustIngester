use rust_ingester::api::routes;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_ingester=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get port from environment or use default
    let port = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("🚀 RustIngester Service starting on {}", addr);
    tracing::info!("📊 Endpoints:");
    tracing::info!("   GET  /status");
    tracing::info!("   POST /ingest/session");
    tracing::info!("   POST /ingest/batch");
    tracing::info!("   POST /ingest/messages");
    tracing::info!("   POST /ingest/knowledge-graph");
    tracing::info!("   GET  /ingest/statistics");
    tracing::info!("   POST /query/similar");
    tracing::info!("   GET  /query/session/:session_id");
    tracing::info!("   POST /query/llm-context");
    tracing::info!("   POST /query/messages");
    tracing::info!("   POST /graph/cypher");

    // Create router
    let app = routes::create_router();

    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("✅ Server listening on {}", addr);
    
    axum::serve(listener, app).await.unwrap();
}
