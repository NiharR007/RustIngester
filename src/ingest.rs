use rand::seq::SliceRandom;
use anyhow::Result;
use crate::db;
use crate::{config::Config, etl::{embed, lsh::Lsh, parser::{ParsedTriplet, SessionGraph, KnowledgeGraphData}}};
use std::collections::HashMap;

/// Quickly seed 100 sample nodes (label Person) and 200 random edges between them.
pub async fn seed_sample_graph() -> Result<()> {
    let client = db::connect::get_client().await?;
    let mut ids = Vec::new();
    // insert nodes
    for i in 0..100 {
        let node_props = serde_json::json!({"name": format!("Person{i}")});
        let id = db::graph::upsert_node(&client, "Person", &format!("person_{i}"), &node_props).await?;
        ids.push(id);
    }
    // random edges
    let mut rng = rand::thread_rng();
    for _ in 0..200 {
        let a = *ids.choose(&mut rng).unwrap();
        let b = *ids.choose(&mut rng).unwrap();
        if a == b {continue;}
        let edge_props = serde_json::Value::Null;
        db::graph::upsert_edge(&client, "KNOWS", a, b, &edge_props).await?;
    }
    Ok(())
}

pub async fn ingest_triplet(t: ParsedTriplet) -> Result<()> {
    let cfg = Config::from_env();
    let client = db::connect::get_client().await?;

    // upsert subject and object nodes
    let subject_id = db::graph::upsert_node(&client, &t.subject.label, &t.subject.pk, &t.subject.props).await?;
    let object_id = db::graph::upsert_node(&client, &t.object.label, &t.object.pk, &t.object.props).await?;

    // upsert edge between them
    db::graph::upsert_edge(&client, &t.relationship, subject_id, object_id, &t.edge_props).await?;

    // Compute embedding and store
    let text = format!("{} {} {}", t.subject.pk, t.relationship, t.object.pk);
    let vec_f32 = embed::embed_text(&text).await?;
    let lsh = Lsh::new(vec_f32.len(), cfg.lsh_buckets);
    let bucket = lsh.hash(&vec_f32) as i32;
    db::vector::upsert_embedding(&client, t.id, &vec_f32, bucket).await?;

    Ok(())
}

// ============================================================================
// New Functions for ok.json Batch Ingestion
// ============================================================================

#[derive(Debug, Clone)]
pub struct SessionIngestStats {
    pub session_id: String,
    pub nodes_created: usize,
    pub edges_created: usize,
    pub embeddings_created: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct BatchIngestStats {
    pub total_sessions: usize,
    pub total_nodes: usize,
    pub total_edges: usize,
    pub total_embeddings: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

/// Ingest a single session graph
pub async fn ingest_session_graph(
    session_id: &str,
    graph: &SessionGraph,
) -> Result<SessionIngestStats> {
    let start = std::time::Instant::now();
    let cfg = Config::from_env();
    let client = db::connect::get_client().await?;
    
    let mut node_map: HashMap<String, i64> = HashMap::new();
    let mut nodes_created = 0;
    let mut edges_created = 0;
    let mut embeddings_created = 0;
    
    // Step 1: Create all nodes
    for node in &graph.nodes {
        let parsed_node = node.to_parsed_node();
        let node_id = db::graph::upsert_node(
            &client,
            &parsed_node.label,
            &parsed_node.pk,
            &parsed_node.props,
        ).await?;
        node_map.insert(node.id.clone(), node_id);
        nodes_created += 1;
    }
    
    // Step 2: Create all edges with evidence tracking
    for (idx, edge) in graph.edges.iter().enumerate() {
        let source_id = node_map.get(&edge.source)
            .ok_or_else(|| anyhow::anyhow!("Source node not found: {}", edge.source))?;
        let target_id = node_map.get(&edge.target)
            .ok_or_else(|| anyhow::anyhow!("Target node not found: {}", edge.target))?;
        
        let edge_props = edge.to_edge_props();
        db::graph::upsert_edge(&client, &edge.relation, *source_id, *target_id, &edge_props).await?;
        edges_created += 1;
        
        // Generate unique edge ID for this session
        // Use a hash of session_id + idx to create a unique i64
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        session_id.hash(&mut hasher);
        idx.hash(&mut hasher);
        let edge_id = hasher.finish() as i64;
        
        // Store evidence
        db::vector::store_edge_evidence(&client, edge_id, session_id, &edge.evidence_message_ids).await?;
        
        // Generate embedding for the edge
        let edge_text = format!("{} {} {}", edge.source, edge.relation, edge.target);
        eprintln!("   Generating embedding for edge {}/{}: {}", idx + 1, graph.edges.len(), edge_text);
        
        let vec_f32 = match embed::embed_text(&edge_text).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("   ❌ Failed to generate embedding: {}", e);
                return Err(e);
            }
        };
        
        let lsh = Lsh::new(vec_f32.len(), cfg.lsh_buckets);
        let bucket = lsh.hash(&vec_f32) as i32;
        
        match db::vector::upsert_embedding_with_session(
            &client,
            edge_id,
            &vec_f32,
            bucket,
            session_id,
            &edge_text,
        ).await {
            Ok(_) => {
                eprintln!("   ✅ Stored embedding for edge {}", idx + 1);
                embeddings_created += 1;
            }
            Err(e) => {
                eprintln!("   ❌ Failed to store embedding: {}", e);
                return Err(e);
            }
        }
    }
    
    // Step 3: Update session metadata
    client.execute(
        "INSERT INTO ag_catalog.sessions(session_id, node_count, edge_count) VALUES($1, $2, $3)
         ON CONFLICT (session_id) DO UPDATE SET 
            node_count = EXCLUDED.node_count,
            edge_count = EXCLUDED.edge_count,
            ingested_at = NOW()",
        &[&session_id, &(nodes_created as i32), &(edges_created as i32)],
    ).await?;
    
    let duration_ms = start.elapsed().as_millis() as u64;
    
    Ok(SessionIngestStats {
        session_id: session_id.to_string(),
        nodes_created,
        edges_created,
        embeddings_created,
        duration_ms,
    })
}

/// Ingest entire knowledge graph data (ok.json format)
pub async fn ingest_knowledge_graph_data(
    data: &KnowledgeGraphData,
) -> Result<BatchIngestStats> {
    let start = std::time::Instant::now();
    let mut total_nodes = 0;
    let mut total_edges = 0;
    let mut total_embeddings = 0;
    let mut errors = Vec::new();
    
    for (session_id, graph) in data {
        match ingest_session_graph(session_id, graph).await {
            Ok(stats) => {
                total_nodes += stats.nodes_created;
                total_edges += stats.edges_created;
                total_embeddings += stats.embeddings_created;
                println!("✓ Ingested session {}: {} nodes, {} edges", 
                    session_id, stats.nodes_created, stats.edges_created);
            }
            Err(e) => {
                let error_msg = format!("Failed to ingest session {}: {:?}", session_id, e);
                eprintln!("✗ {}", error_msg);
                errors.push(error_msg);
            }
        }
    }
    
    let duration_ms = start.elapsed().as_millis() as u64;
    
    Ok(BatchIngestStats {
        total_sessions: data.len(),
        total_nodes,
        total_edges,
        total_embeddings,
        duration_ms,
        errors,
    })
}

/// Load and ingest from a JSON file
pub async fn ingest_from_file(file_path: &str) -> Result<BatchIngestStats> {
    let content = tokio::fs::read_to_string(file_path).await?;
    let data: KnowledgeGraphData = serde_json::from_str(&content)?;
    ingest_knowledge_graph_data(&data).await
}
