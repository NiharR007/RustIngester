use rand::seq::SliceRandom;
use anyhow::Result;
use crate::db;
use crate::{config::Config, etl::{embed, lsh::Lsh, parser::ParsedTriplet}};

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
