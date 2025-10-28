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
            "INSERT INTO embeddings(triplet_id, vec, lsh_bucket) VALUES($1, $2, $3)
             ON CONFLICT (triplet_id) DO UPDATE SET vec = EXCLUDED.vec, lsh_bucket = EXCLUDED.lsh_bucket",
            &[&triplet_id, &vec_json, &bucket],
        )
        .await?;
    Ok(())
}
