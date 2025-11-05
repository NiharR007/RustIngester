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

    // Ensure pgvector extension exists (for message embeddings)
    client
        .batch_execute(
            "CREATE EXTENSION IF NOT EXISTS vector;"
        )
        .await?;

    // Try to ensure AGE extension and graph exist (optional - for knowledge graph features)
    let age_result = client
        .batch_execute(
            "CREATE EXTENSION IF NOT EXISTS age;\nLOAD 'age';\nSET search_path = ag_catalog, \"$user\", public;"
        )
        .await;
    
    if age_result.is_ok() {
        eprintln!("✅ AGE extension loaded successfully");
        
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
        let _ = client
            .batch_execute(
                "SELECT create_graph('sem_graph') WHERE NOT EXISTS (SELECT 1 FROM ag_graph WHERE name='sem_graph');"
            )
            .await;
    } else {
        eprintln!("⚠️  AGE extension not available - knowledge graph features will be limited");
    }
    
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
    
    // Run message and knowledge graph schema migration
    run_message_schema_migration(&client).await?;

    Ok(client)
}

/// Run the message and knowledge graph schema migration
async fn run_message_schema_migration(client: &Client) -> Result<()> {
    println!("Running message schema migration...");

    // Enable UUID extension
    client.execute("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";", &[]).await?;

    // Conversations table
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS conversations (
            conversation_id UUID PRIMARY KEY,
            created_at TIMESTAMP DEFAULT NOW(),
            updated_at TIMESTAMP DEFAULT NOW(),
            metadata JSONB DEFAULT '{}'::jsonb
        );"
    ).await?;

    // Messages table
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS messages (
            message_id UUID PRIMARY KEY,
            conversation_id UUID NOT NULL REFERENCES conversations(conversation_id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT NOW(),
            metadata JSONB DEFAULT '{}'::jsonb
        );"
    ).await?;

    // Message embeddings with pgvector
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS message_embeddings (
            message_id UUID PRIMARY KEY REFERENCES messages(message_id) ON DELETE CASCADE,
            embedding vector(768) NOT NULL,
            embedding_model VARCHAR(100) DEFAULT 'nomic-embed-text-v1.5',
            created_at TIMESTAMP DEFAULT NOW()
        );"
    ).await?;

    // Knowledge graph nodes
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS kg_nodes (
            node_id VARCHAR(255),
            conversation_id UUID REFERENCES conversations(conversation_id) ON DELETE CASCADE,
            node_type VARCHAR(100),
            created_at TIMESTAMP DEFAULT NOW(),
            PRIMARY KEY (node_id, conversation_id)
        );"
    ).await?;

    // Knowledge graph edges
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS kg_edges (
            edge_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            conversation_id UUID REFERENCES conversations(conversation_id) ON DELETE CASCADE,
            source_node VARCHAR(255) NOT NULL,
            target_node VARCHAR(255) NOT NULL,
            relation VARCHAR(255) NOT NULL,
            evidence_message_ids UUID[] NOT NULL,
            created_at TIMESTAMP DEFAULT NOW()
        );"
    ).await?;

    // Create indexes
    client.batch_execute(
        "CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id);
         CREATE INDEX IF NOT EXISTS idx_message_embeddings_ivfflat ON message_embeddings
             USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
         CREATE INDEX IF NOT EXISTS idx_kg_edges_conversation ON kg_edges(conversation_id);
         CREATE INDEX IF NOT EXISTS idx_kg_edges_evidence ON kg_edges USING GIN(evidence_message_ids);
         CREATE INDEX IF NOT EXISTS idx_kg_nodes_conversation ON kg_nodes(conversation_id);
         CREATE INDEX IF NOT EXISTS idx_kg_nodes_type ON kg_nodes(node_type);"
    ).await?;

    println!("Message schema migration completed successfully");
    Ok(())
}

