use tokio_postgres::{Client, Error};
use uuid::Uuid;
use pgvector::Vector;
use crate::db::models::*;

/// Insert or update a conversation record
pub async fn insert_conversation(
    client: &Client,
    conversation_id: Uuid,
) -> Result<(), Error> {
    client.execute(
        "INSERT INTO conversations (conversation_id) 
         VALUES ($1) 
         ON CONFLICT (conversation_id) DO NOTHING",
        &[&conversation_id],
    ).await?;
    Ok(())
}

/// Insert a message with its embedding
pub async fn insert_message_with_embedding(
    client: &Client,
    turn_data: &TurnEmbedding,
) -> Result<(), Error> {
    // Insert message
    client.execute(
        "INSERT INTO messages (message_id, conversation_id, content)
         VALUES ($1, $2, $3)
         ON CONFLICT (message_id) DO UPDATE 
         SET content = EXCLUDED.content",
        &[
            &turn_data.message_id,
            &turn_data.conversation_id,
            &turn_data.actual_text,
        ],
    ).await?;

    // Convert embedding Vec<f32> to pgvector Vector type
    let embedding_vec = Vector::from(turn_data.embedding.clone());

    // Insert embedding
    client.execute(
        "INSERT INTO message_embeddings (message_id, embedding)
         VALUES ($1, $2)
         ON CONFLICT (message_id) DO UPDATE 
         SET embedding = EXCLUDED.embedding",
        &[&turn_data.message_id, &embedding_vec],
    ).await?;

    Ok(())
}

/// Batch insert messages and embeddings
pub async fn batch_insert_messages(
    client: &Client,
    turns: &[TurnEmbedding],
) -> Result<(usize, Vec<String>), Error> {
    let mut success_count = 0;
    let mut errors = Vec::new();

    // Collect unique conversation IDs
    let mut conv_ids: Vec<Uuid> = turns.iter()
        .map(|t| t.conversation_id)
        .collect();
    conv_ids.sort();
    conv_ids.dedup();

    // Insert all conversations first
    for conv_id in conv_ids {
        insert_conversation(client, conv_id).await?;
    }

    // Insert messages and embeddings
    for turn in turns {
        match insert_message_with_embedding(client, turn).await {
            Ok(_) => success_count += 1,
            Err(e) => {
                errors.push(format!("Message {}: {}", turn.message_id, e));
                eprintln!("Failed to insert message {}: {}", turn.message_id, e);
            }
        }
    }

    Ok((success_count, errors))
}

/// Retrieve messages by their IDs, maintaining the order of input IDs
pub async fn get_messages_by_ids_ordered(
    client: &Client,
    message_ids: &[Uuid],
) -> Result<Vec<Message>, Error> {
    if message_ids.is_empty() {
        return Ok(Vec::new());
    }

    let rows = client.query(
        "SELECT m.message_id, m.conversation_id, m.content
         FROM messages m
         WHERE m.message_id = ANY($1::uuid[])
         ORDER BY array_position($1::uuid[], m.message_id)",
        &[&message_ids],
    ).await?;

    let messages = rows.iter().map(|row| Message {
        message_id: row.get(0),
        conversation_id: row.get(1),
        content: row.get(2),
    }).collect();

    Ok(messages)
}

/// Get messages with their similarity scores based on embedding similarity to a query
pub async fn get_similar_messages_by_embedding(
    client: &Client,
    query_embedding: &[f32],
    limit: i64,
) -> Result<Vec<MessageWithRelevance>, Error> {
    let embedding_vec = Vector::from(query_embedding.to_vec());

    let rows = client.query(
        "SELECT m.message_id, m.conversation_id, m.content,
                1 - (me.embedding <=> $1) as similarity
         FROM ag_catalog.messages m
         JOIN ag_catalog.message_embeddings me ON m.message_id = me.message_id
         ORDER BY me.embedding <=> $1
         LIMIT $2",
        &[&embedding_vec, &limit],
    ).await?;

    let messages = rows.iter().map(|row| {
        let similarity: f64 = row.get(3);
        MessageWithRelevance {
            message_id: row.get(0),
            conversation_id: row.get(1),
            content: row.get(2),
            relevance_score: similarity as f32,
        }
    }).collect();

    Ok(messages)
}

