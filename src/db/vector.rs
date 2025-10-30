use anyhow::Result;
use tokio_postgres::Client;

/// Upsert embedding vector row (storing as JSON text for now).
pub async fn upsert_embedding(
    client: &Client,
    triplet_id: i64,
    vec: &[f32],
    bucket: i32,
) -> Result<()> {
    let vec_json = serde_json::to_string(vec)?;
    client
        .execute(
            "INSERT INTO ag_catalog.embeddings(triplet_id, vec, lsh_bucket) VALUES($1, $2, $3)
             ON CONFLICT (triplet_id) DO UPDATE SET vec = EXCLUDED.vec, lsh_bucket = EXCLUDED.lsh_bucket",
            &[&triplet_id, &vec_json, &bucket],
        )
        .await?;
    Ok(())
}

/// Upsert embedding with session tracking
pub async fn upsert_embedding_with_session(
    client: &Client,
    triplet_id: i64,
    vec: &[f32],
    bucket: i32,
    session_id: &str,
    edge_text: &str,
) -> Result<()> {
    let vec_json = serde_json::to_string(vec)?;
    client
        .execute(
            "INSERT INTO ag_catalog.embeddings(triplet_id, vec, lsh_bucket, session_id, edge_text) 
             VALUES($1, $2, $3, $4, $5)
             ON CONFLICT (triplet_id) DO UPDATE SET 
                vec = EXCLUDED.vec, 
                lsh_bucket = EXCLUDED.lsh_bucket,
                session_id = EXCLUDED.session_id,
                edge_text = EXCLUDED.edge_text",
            &[&triplet_id, &vec_json, &bucket, &session_id, &edge_text],
        )
        .await?;
    Ok(())
}

/// Store evidence for an edge
pub async fn store_edge_evidence(
    client: &Client,
    edge_id: i64,
    session_id: &str,
    evidence_ids: &[String],
) -> Result<()> {
    for evidence_id in evidence_ids {
        client
            .execute(
                "INSERT INTO ag_catalog.edge_evidence(edge_id, session_id, evidence_message_id) 
                 VALUES($1, $2, $3)
                 ON CONFLICT DO NOTHING",
                &[&edge_id, &session_id, &evidence_id],
            )
            .await?;
    }
    Ok(())
}
