use anyhow::Result;
use tokio_postgres::{Client, NoTls};

use crate::config::Config;

/// Obtain a connected `tokio_postgres::Client` and spawn the connection task.
pub async fn get_client() -> Result<Client> {
    let cfg = Config::from_env();
    let (client, connection) = tokio_postgres::connect(&cfg.db_url, NoTls).await?;
    // Drive the connection on a background task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {e}");
        }
    });
    // Ensure AGE extension and graph exist
    client
        .batch_execute(
            "CREATE EXTENSION IF NOT EXISTS age;\nLOAD 'age';\nSET search_path = ag_catalog, \"$user\", public;"
        )
        .await?;
    
    // Create common vertex labels if they don't exist
    let labels = vec!["Node", "TestNode", "Person", "City"];
    for label in labels {
        let create_label_sql = format!(
            "SELECT ag_catalog.create_vlabel('sem_graph', '{}') WHERE NOT EXISTS (SELECT 1 FROM ag_catalog.ag_label WHERE name='{}' AND graph=17033);",
            label, label
        );
        let _ = client.execute(&create_label_sql, &[]).await; // Ignore errors if label exists
    }
    // create graph if not exists
    client
        .batch_execute(
            "SELECT create_graph('sem_graph') WHERE NOT EXISTS (SELECT 1 FROM ag_graph WHERE name='sem_graph');"
        )
        .await?;
    
    // Create embeddings table if it doesn't exist (explicitly in ag_catalog schema)
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS ag_catalog.embeddings (
                 triplet_id BIGINT PRIMARY KEY,
                 vec TEXT,
                 lsh_bucket INTEGER,
                 session_id TEXT,
                 edge_text TEXT
             );"
        )
        .await?;
    
    // Create sessions metadata table (explicitly in ag_catalog schema)
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS ag_catalog.sessions (
                 session_id TEXT PRIMARY KEY,
                 ingested_at TIMESTAMP DEFAULT NOW(),
                 node_count INTEGER,
                 edge_count INTEGER
             );"
        )
        .await?;
    
    // Create edge evidence tracking table (explicitly in ag_catalog schema)
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS ag_catalog.edge_evidence (
                 edge_id BIGINT,
                 session_id TEXT,
                 evidence_message_id TEXT,
                 PRIMARY KEY (edge_id, evidence_message_id)
             );"
        )
        .await?;
    
    Ok(client)
}
