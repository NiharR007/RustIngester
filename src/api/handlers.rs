use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use crate::api::models::*;
use crate::db;
use crate::ingest;

/// Health check endpoint
pub async fn health_check() -> Result<Json<StatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    match get_system_status().await {
        Ok(status) => Ok(Json(status)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("health_check_failed", e.to_string())),
        )),
    }
}

async fn get_system_status() -> anyhow::Result<StatusResponse> {
    let client = db::connect::get_client().await?;
    
    // Get session count (explicitly use ag_catalog schema)
    let session_count = client
        .query_one("SELECT COUNT(*) FROM ag_catalog.sessions", &[])
        .await?
        .get::<_, i64>(0);
    
    // Get total nodes (approximate from AGE)
    let node_count_result = client
        .query("SELECT COUNT(*) FROM ag_catalog.ag_label WHERE graph = (SELECT graphid FROM ag_catalog.ag_graph WHERE name = 'sem_graph')", &[])
        .await;
    let node_count = node_count_result.map(|rows| rows.len() as i64).unwrap_or(0);
    
    // Get total embeddings (explicitly use ag_catalog schema)
    let edge_count = client
        .query_one("SELECT COUNT(*) FROM ag_catalog.embeddings", &[])
        .await?
        .get::<_, i64>(0);
    
    Ok(StatusResponse {
        status: "healthy".to_string(),
        database: "connected".to_string(),
        age_extension: "loaded".to_string(),
        graph_name: "sem_graph".to_string(),
        total_sessions: session_count,
        total_nodes: node_count,
        total_edges: edge_count,
    })
}

/// Ingest a single session graph
pub async fn ingest_session(
    Json(payload): Json<IngestSessionRequest>,
) -> Result<Json<IngestSessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    match ingest::ingest_session_graph(&payload.session_id, &payload.graph).await {
        Ok(stats) => Ok(Json(stats.into())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("ingestion_failed", e.to_string())),
        )),
    }
}

/// Ingest batch of sessions
pub async fn ingest_batch(
    Json(payload): Json<IngestBatchRequest>,
) -> Result<Json<IngestBatchResponse>, (StatusCode, Json<ErrorResponse>)> {
    match ingest::ingest_knowledge_graph_data(&payload.sessions).await {
        Ok(stats) => Ok(Json(stats.into())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("batch_ingestion_failed", e.to_string())),
        )),
    }
}

/// Query similar edges
pub async fn query_similar(
    Json(payload): Json<QuerySimilarRequest>,
) -> Result<Json<QuerySimilarResponse>, (StatusCode, Json<ErrorResponse>)> {
    match query_similar_edges(&payload.query, payload.top_k, payload.threshold).await {
        Ok(results) => Ok(Json(QuerySimilarResponse {
            count: results.len(),
            results,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse::new("query_failed", e.to_string())),
        )),
    }
}

async fn query_similar_edges(
    query: &str,
    top_k: i64,
    threshold: Option<f32>,
) -> anyhow::Result<Vec<SimilarityResult>> {
    use crate::{config::Config, etl::{embed, lsh::Lsh}};
    
    let cfg = Config::from_env();
    let client = db::connect::get_client().await?;
    
    // Generate query embedding
    let query_vec = embed::embed_text(query).await?;
    let lsh = Lsh::new(query_vec.len(), cfg.lsh_buckets);
    let bucket = lsh.hash(&query_vec) as i32;
    
    eprintln!("🔍 Query similarity search (API handler):");
    eprintln!("   Query text: {}", query);
    eprintln!("   Query bucket: {}", bucket);
    eprintln!("   LSH buckets config: {}", cfg.lsh_buckets);
    
    // Get all vectors in the same LSH bucket with session info
    let sql = "SELECT triplet_id, vec, session_id, edge_text FROM embeddings WHERE lsh_bucket = $1";
    let rows = client.query(sql, &[&bucket]).await?;
    
    eprintln!("   Found {} embeddings in bucket {}", rows.len(), bucket);
    
    // If no results in the specific bucket, fall back to searching all embeddings
    let rows = if rows.is_empty() {
        eprintln!("   ⚠️  Bucket {} is empty, searching ALL embeddings as fallback", bucket);
        let sql_all = "SELECT triplet_id, vec, session_id, edge_text FROM embeddings LIMIT 1000";
        let all_rows = client.query(sql_all, &[]).await?;
        eprintln!("   Found {} total embeddings in database", all_rows.len());
        
        // Show bucket distribution
        let bucket_count_sql = "SELECT lsh_bucket, COUNT(*) FROM embeddings GROUP BY lsh_bucket ORDER BY lsh_bucket";
        let bucket_rows = client.query(bucket_count_sql, &[]).await?;
        eprintln!("   Bucket distribution:");
        for br in bucket_rows.iter().take(10) {
            let b: i32 = br.get(0);
            let count: i64 = br.get(1);
            eprintln!("      Bucket {}: {} embeddings", b, count);
        }
        if bucket_rows.len() > 10 {
            eprintln!("      ... and {} more buckets", bucket_rows.len() - 10);
        }
        
        all_rows
    } else {
        rows
    };
    
    let mut results = Vec::new();
    for row in rows {
        let triplet_id: i64 = row.get(0);
        let vec_json: String = row.get(1);
        let session_id: Option<String> = row.get(2);
        let edge_text: Option<String> = row.get(3);
        
        let stored_vec: Vec<f32> = serde_json::from_str(&vec_json)?;
        
        // Calculate similarity
        let similarity = cosine_similarity(&query_vec, &stored_vec);
        let distance = 1.0 - similarity;
        
        // Apply threshold if specified
        if let Some(thresh) = threshold {
            if similarity < thresh {
                continue;
            }
        }
        
        // Get evidence for this edge
        let evidence_rows = client
            .query(
                "SELECT evidence_message_id FROM edge_evidence WHERE edge_id = $1",
                &[&triplet_id],
            )
            .await?;
        
        let evidence_message_ids: Vec<String> = evidence_rows
            .iter()
            .map(|r| r.get::<_, String>(0))
            .collect();
        
        // Parse edge text (format: "source relation target")
        let edge = if let Some(text) = edge_text {
            parse_edge_text(&text)
        } else {
            EdgeResult {
                source: "unknown".to_string(),
                relation: "unknown".to_string(),
                target: "unknown".to_string(),
            }
        };
        
        results.push(SimilarityResult {
            session_id: session_id.unwrap_or_else(|| "unknown".to_string()),
            edge,
            similarity,
            distance,
            evidence_message_ids,
        });
    }
    
    // Sort by similarity (descending) and take top k
    results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    results.truncate(top_k as usize);
    
    eprintln!("   Returning {} results", results.len());
    if !results.is_empty() {
        eprintln!("   Top result: similarity={:.4}, edge={} {} {}", 
            results[0].similarity,
            results[0].edge.source,
            results[0].edge.relation,
            results[0].edge.target
        );
    }
    
    Ok(results)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

fn parse_edge_text(text: &str) -> EdgeResult {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() >= 3 {
        EdgeResult {
            source: parts[0].to_string(),
            relation: parts[1].to_string(),
            target: parts[2..].join(" "),
        }
    } else {
        EdgeResult {
            source: "unknown".to_string(),
            relation: "unknown".to_string(),
            target: "unknown".to_string(),
        }
    }
}

/// Get session graph by ID
pub async fn get_session(
    Path(session_id): Path<String>,
) -> Result<Json<SessionGraphResponse>, (StatusCode, Json<ErrorResponse>)> {
    match retrieve_session_graph(&session_id).await {
        Ok(graph) => Ok(Json(SessionGraphResponse { session_id, graph })),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse::new("session_not_found", e.to_string())),
        )),
    }
}

async fn retrieve_session_graph(_session_id: &str) -> anyhow::Result<crate::etl::parser::SessionGraph> {
    // This is a placeholder - you would need to implement graph retrieval from AGE
    // For now, return an empty graph
    Ok(crate::etl::parser::SessionGraph {
        nodes: vec![],
        edges: vec![],
    })
}

/// Execute custom Cypher query
pub async fn execute_cypher(
    Json(payload): Json<CypherQueryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    match run_cypher_query(&payload.query).await {
        Ok(results) => Ok(Json(serde_json::json!({
            "results": results,
            "count": results.as_array().map(|a| a.len()).unwrap_or(0)
        }))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new("cypher_query_failed", e.to_string())),
        )),
    }
}

async fn run_cypher_query(query: &str) -> anyhow::Result<serde_json::Value> {
    let client = db::connect::get_client().await?;
    
    let cypher = format!(
        "SELECT * FROM ag_catalog.cypher('sem_graph'::name, $$
         {}
         $$::cstring) AS (result ag_catalog.agtype);",
        query
    );
    
    let rows = client.query(&cypher, &[]).await?;
    let results: Vec<String> = rows.iter().map(|r| r.get::<_, String>(0)).collect();
    
    Ok(serde_json::json!(results))
}
