use anyhow::Result;

use crate::{config::Config, db, etl::{embed, lsh::Lsh}};

/// Simple cosine similarity calculation for demonstration
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

pub async fn query_similar(text: &str, k: i64) -> Result<Vec<(i64, f32)>> {
    let cfg = Config::from_env();
    let client = db::connect::get_client().await?;

    let query_vec = embed::embed_text(text).await?;
    let lsh = Lsh::new(query_vec.len(), cfg.lsh_buckets);
    let bucket = lsh.hash(&query_vec) as i32;

    eprintln!("🔍 Query similarity search:");
    eprintln!("   Query text: {}", text);
    eprintln!("   Query bucket: {}", bucket);
    eprintln!("   LSH buckets config: {}", cfg.lsh_buckets);

    // Get all vectors in the same LSH bucket
    let sql = "SELECT triplet_id, vec FROM embeddings WHERE lsh_bucket = $1";
    let rows = client.query(sql, &[&bucket]).await?;
    
    eprintln!("   Found {} embeddings in bucket {}", rows.len(), bucket);
    
    // If no results in the specific bucket, fall back to searching all embeddings
    let rows = if rows.is_empty() {
        eprintln!("   ⚠️  Bucket {} is empty, searching ALL embeddings as fallback", bucket);
        let sql_all = "SELECT triplet_id, vec, lsh_bucket FROM embeddings LIMIT 1000";
        let all_rows = client.query(sql_all, &[]).await?;
        eprintln!("   Found {} total embeddings in database", all_rows.len());
        
        // Show bucket distribution
        let bucket_count_sql = "SELECT lsh_bucket, COUNT(*) FROM embeddings GROUP BY lsh_bucket ORDER BY lsh_bucket";
        let bucket_rows = client.query(bucket_count_sql, &[]).await?;
        eprintln!("   Bucket distribution:");
        for br in bucket_rows {
            let b: i32 = br.get(0);
            let count: i64 = br.get(1);
            eprintln!("      Bucket {}: {} embeddings", b, count);
        }
        
        all_rows
    } else {
        rows
    };
    
    let mut results = Vec::new();
    for row in rows {
        let triplet_id: i64 = row.get(0);
        let vec_json: String = row.get(1);
        let stored_vec: Vec<f32> = serde_json::from_str(&vec_json)?;
        
        // Calculate similarity (convert to distance: 1 - similarity)
        let similarity = cosine_similarity(&query_vec, &stored_vec);
        let distance = 1.0 - similarity;
        results.push((triplet_id, distance));
    }
    
    // Sort by distance (ascending) and take top k
    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    results.truncate(k as usize);
    
    eprintln!("   Returning {} results", results.len());
    if !results.is_empty() {
        eprintln!("   Top result: triplet_id={}, distance={:.4}", results[0].0, results[0].1);
    }
    
    Ok(results)
}
