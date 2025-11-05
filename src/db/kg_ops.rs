use tokio_postgres::{Client, Error};
use uuid::Uuid;
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
pub async fn insert_kg_edge(
    client: &Client,
    conversation_id: Uuid,
    edge: &KGEdge,
) -> Result<(), Error> {
    client.execute(
        "INSERT INTO kg_edges (conversation_id, source_node, target_node, relation, evidence_message_ids)
         VALUES ($1, $2, $3, $4, $5)",
        &[
            &conversation_id,
            &edge.source,
            &edge.target,
            &edge.relation,
            &edge.evidence_message_ids,
        ],
    ).await?;
    Ok(())
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

        // Insert edges
        for edge in &kg.edges {
            match insert_kg_edge(client, conversation_id, edge).await {
                Ok(_) => total_edges += 1,
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

