use tokio_postgres::{Client, Error};
use uuid::Uuid;
use pgvector::Vector;
use crate::db::models::*;
use crate::db::message_ops::insert_conversation;

/// Insert a knowledge graph node
pub async fn insert_kg_node(
    client: &Client,
    conversation_id: Uuid,
    node: &KGNode,
) -> Result<(), Error> {
    client.execute(
        "INSERT INTO kg_nodes (node_id, conversation_id, node_type)
         VALUES ($1, $2, $3)
         ON CONFLICT (node_id, conversation_id) DO UPDATE
         SET node_type = EXCLUDED.node_type",
        &[&node.id, &conversation_id, &node.node_type],
    ).await?;
    Ok(())
}

/// Insert a knowledge graph edge with evidence
/// Returns the edge_id of the inserted edge
pub async fn insert_kg_edge(
    client: &Client,
    conversation_id: Uuid,
    edge: &KGEdge,
) -> Result<Uuid, Error> {
    let row = client.query_one(
        "INSERT INTO kg_edges (conversation_id, source_node, target_node, relation, evidence_message_ids)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING edge_id",
        &[
            &conversation_id,
            &edge.source,
            &edge.target,
            &edge.relation,
            &edge.evidence_message_ids,
        ],
    ).await?;
    
    let edge_id: Uuid = row.get(0);
    Ok(edge_id)
}

/// Batch insert knowledge graph data for multiple conversations
pub async fn batch_insert_knowledge_graph(
    client: &Client,
    kg_data: ConversationKnowledgeGraph,
) -> Result<(usize, usize, Vec<String>), Error> {
    let mut total_nodes = 0;
    let mut total_edges = 0;
    let mut errors = Vec::new();

    for (conversation_id, kg) in kg_data.conversations {
        // Ensure conversation exists
        if let Err(e) = insert_conversation(client, conversation_id).await {
            errors.push(format!("Conversation {}: {}", conversation_id, e));
            continue;
        }

        // Insert nodes
        for node in &kg.nodes {
            match insert_kg_node(client, conversation_id, node).await {
                Ok(_) => total_nodes += 1,
                Err(e) => {
                    errors.push(format!("Node {} in conv {}: {}", node.id, conversation_id, e));
                    eprintln!("Failed to insert node {} in conversation {}: {}",
                        node.id, conversation_id, e);
                }
            }
        }

        // Insert edges and generate embeddings
        for edge in &kg.edges {
            match insert_kg_edge(client, conversation_id, edge).await {
                Ok(edge_id) => {
                    total_edges += 1;
                    
                    // Generate edge text for embedding: "source relation target"
                    let edge_text = format!("{} {} {}", edge.source, edge.relation, edge.target);
                    
                    // Generate embedding using llama.cpp
                    use crate::etl::embed;
                    match embed::embed_text(&edge_text).await {
                        Ok(embedding) => {
                            // Insert the edge embedding
                            if let Err(e) = insert_kg_edge_embedding(client, edge_id, &embedding, &edge_text).await {
                                errors.push(format!("Embedding for edge {}->{}: {}", 
                                    edge.source, edge.target, e));
                                eprintln!("Failed to insert embedding for edge {}->{}: {}",
                                    edge.source, edge.target, e);
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Failed to generate embedding for edge {}->{}: {}",
                                edge.source, edge.target, e));
                            eprintln!("Failed to generate embedding for edge {}->{}: {}",
                                edge.source, edge.target, e);
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Edge {}->{} in conv {}: {}",
                        edge.source, edge.target, conversation_id, e));
                    eprintln!("Failed to insert edge {}->{} in conversation {}: {}",
                        edge.source, edge.target, conversation_id, e);
                }
            }
        }
    }

    Ok((total_nodes, total_edges, errors))
}

/// Query knowledge graph edges by keyword matching
pub async fn get_edges_by_query(
    client: &Client,
    query_keywords: &[String],
    limit: i64,
) -> Result<Vec<KGEdgeWithContext>, Error> {
    if query_keywords.is_empty() {
        return Ok(Vec::new());
    }

    // Convert keywords to ILIKE patterns
    let patterns: Vec<String> = query_keywords.iter()
        .map(|kw| format!("%{}%", kw))
        .collect();

    let rows = client.query(
        "SELECT DISTINCT conversation_id, source_node, target_node, relation, evidence_message_ids
         FROM kg_edges
         WHERE source_node ILIKE ANY($1)
            OR target_node ILIKE ANY($1)
            OR relation ILIKE ANY($1)
         LIMIT $2",
        &[&patterns, &limit],
    ).await?;

    let edges = rows.iter().map(|row| KGEdgeWithContext {
        conversation_id: row.get(0),
        source: row.get(1),
        target: row.get(2),
        relation: row.get(3),
        evidence_message_ids: row.get(4),
    }).collect();

    Ok(edges)
}

/// Get all edges that contain any of the specified message IDs in their evidence
pub async fn get_edges_by_message_ids(
    client: &Client,
    message_ids: &[Uuid],
) -> Result<Vec<KGEdgeWithContext>, Error> {
    if message_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = client.query(
        "SELECT conversation_id, source_node, target_node, relation, evidence_message_ids
         FROM kg_edges
         WHERE evidence_message_ids && $1::uuid[]",
        &[&message_ids],
    ).await?;

    let edges = rows.iter().map(|row| KGEdgeWithContext {
        conversation_id: row.get(0),
        source: row.get(1),
        target: row.get(2),
        relation: row.get(3),
        evidence_message_ids: row.get(4),
    }).collect();

    Ok(edges)
}

/// Get statistics about the knowledge graph
pub async fn get_kg_statistics(client: &Client) -> Result<serde_json::Value, Error> {
    let node_count: i64 = client.query_one(
        "SELECT COUNT(*) FROM kg_nodes",
        &[]
    ).await?.get(0);

    let edge_count: i64 = client.query_one(
        "SELECT COUNT(*) FROM kg_edges",
        &[]
    ).await?.get(0);

    let conversation_count: i64 = client.query_one(
        "SELECT COUNT(DISTINCT conversation_id) FROM conversations",
        &[]
    ).await?.get(0);

    let message_count: i64 = client.query_one(
        "SELECT COUNT(*) FROM messages",
        &[]
    ).await?.get(0);

    Ok(serde_json::json!({
        "total_nodes": node_count,
        "total_edges": edge_count,
        "total_conversations": conversation_count,
        "total_messages": message_count,
    }))
}

/// Insert an embedding for a knowledge graph edge
pub async fn insert_kg_edge_embedding(
    client: &Client,
    edge_id: Uuid,
    embedding: &[f32],
    edge_text: &str,
) -> Result<(), Error> {
    let embedding_vec = Vector::from(embedding.to_vec());
    
    client.execute(
        "INSERT INTO ag_catalog.kg_edge_embeddings (edge_id, embedding, edge_text)
         VALUES ($1, $2, $3)
         ON CONFLICT (edge_id) DO UPDATE 
         SET embedding = EXCLUDED.embedding, edge_text = EXCLUDED.edge_text",
        &[&edge_id, &embedding_vec, &edge_text],
    ).await?;
    
    Ok(())
}

/// Get similar edges by embedding similarity (for RAG retrieval)
/// Returns edges with their evidence_message_ids
pub async fn get_similar_edges_by_embedding(
    client: &Client,
    query_embedding: &[f32],
    limit: i64,
) -> Result<Vec<(KGEdgeWithContext, f32)>, Error> {
    let embedding_vec = Vector::from(query_embedding.to_vec());
    
    eprintln!("DEBUG: Searching for similar edges with embedding dim={}, limit={}", 
        query_embedding.len(), limit);

    let rows = client.query(
        "SELECT e.edge_id, e.conversation_id, e.source_node, e.target_node, e.relation, 
                e.evidence_message_ids, 1 - (ee.embedding <=> $1) as similarity
         FROM ag_catalog.kg_edges e
         JOIN ag_catalog.kg_edge_embeddings ee ON e.edge_id = ee.edge_id
         ORDER BY ee.embedding <=> $1
         LIMIT $2",
        &[&embedding_vec, &limit],
    ).await?;
    
    eprintln!("DEBUG: Query returned {} rows", rows.len());

    let edges = rows.iter().map(|row| {
        let similarity: f64 = row.get(6);
        let edge = KGEdgeWithContext {
            conversation_id: row.get(1),
            source: row.get(2),
            target: row.get(3),
            relation: row.get(4),
            evidence_message_ids: row.get(5),
        };
        (edge, similarity as f32)
    }).collect();

    Ok(edges)
}

