use tokio_postgres::{Client, Error};
use uuid::Uuid;
use pgvector::Vector;
use crate::db::models::*;
use std::collections::HashSet;

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

/// Search messages by keyword using PostgreSQL Full-Text Search (BM25-style ranking)
pub async fn search_messages_by_keywords(
    client: &Client,
    keywords: &[String],
    limit: i64,
) -> Result<Vec<MessageWithRelevance>, Error> {
    if keywords.is_empty() {
        return Ok(Vec::new());
    }

    // Build tsquery from keywords (OR them together)
    // Use prefix matching (:*) to handle stemming issues (editdistance → editdist:*)
    let query_parts: Vec<String> = keywords.iter()
        .map(|kw| format!("{}:*", kw))
        .collect();
    let query_string = query_parts.join(" | ");
    
    let rows = client.query(
        "SELECT message_id, conversation_id, content,
                ts_rank(content_tsv, to_tsquery('english', $1), 1) as rank
         FROM ag_catalog.messages 
         WHERE content_tsv @@ to_tsquery('english', $1)
         ORDER BY rank DESC
         LIMIT $2",
        &[&query_string, &limit],
    ).await?;
    
    let messages = rows.iter().map(|row| {
        let rank: f32 = row.get(3);
        MessageWithRelevance {
            message_id: row.get(0),
            conversation_id: row.get(1),
            content: row.get(2),
            relevance_score: rank, // BM25-style rank from ts_rank_cd
        }
    }).collect();
    
    Ok(messages)
}

/// Fallback: Simple ILIKE search for when full-text search fails
pub async fn search_messages_by_keywords_simple(
    client: &Client,
    keywords: &[String],
    limit: i64,
) -> Result<Vec<Message>, Error> {
    if keywords.is_empty() {
        return Ok(Vec::new());
    }

    // Build ILIKE conditions for each keyword
    let patterns: Vec<String> = keywords.iter()
        .map(|kw| format!("%{}%", kw))
        .collect();
    
    let rows = client.query(
        "SELECT message_id, conversation_id, content 
         FROM ag_catalog.messages 
         WHERE content ILIKE ANY($1)
         LIMIT $2",
        &[&patterns, &limit],
    ).await?;
    
    let messages = rows.iter().map(|row| Message {
        message_id: row.get(0),
        conversation_id: row.get(1),
        content: row.get(2),
    }).collect();
    
    Ok(messages)
}

/// Expand query with synonyms and related terms for better BM25 coverage
fn expand_query_keywords(keywords: &[String]) -> Vec<String> {
    let mut expanded = keywords.to_vec();
    
    // Common programming/tech synonyms
    let synonyms: Vec<(&str, Vec<&str>)> = vec![
        ("install", vec!["setup", "installation", "installing", "installed", "pip", "npm", "brew"]),
        ("error", vec!["exception", "bug", "issue", "problem", "fail", "failed", "crash"]),
        ("package", vec!["library", "module", "dependency", "import"]),
        ("function", vec!["method", "def", "procedure", "func"]),
        ("data", vec!["dataset", "information", "records", "dataframe"]),
        ("model", vec!["algorithm", "classifier", "network", "neural"]),
        ("train", vec!["training", "fit", "learn", "learning"]),
        ("test", vec!["testing", "evaluate", "validation", "verify"]),
        ("api", vec!["endpoint", "service", "interface", "rest"]),
        ("database", vec!["db", "storage", "postgres", "sql", "mongodb"]),
    ];
    
    for keyword in keywords {
        let keyword_lower = keyword.to_lowercase();
        for (base, variants) in &synonyms {
            // Only expand if keyword matches the base term (not vice versa)
            // This prevents expanding proper nouns like "Zapier"
            if keyword_lower == *base || keyword_lower.starts_with(base) {
                for variant in variants {
                    if !expanded.iter().any(|k| k.to_lowercase() == *variant) {
                        expanded.push(variant.to_string());
                    }
                }
                break; // Only match once per keyword
            }
        }
    }
    
    expanded
}

/// Hybrid search: Combine keyword search + embedding search with smart prioritization
pub async fn hybrid_search_messages(
    client: &Client,
    query: &str,
    query_embedding: &[f32],
    top_k: i64,
) -> Result<Vec<MessageWithRelevance>, Error> {
    let mut message_ids = HashSet::new();
    let mut results = Vec::new();

    // Strategy 1: Extract meaningful keywords from query
    // Filter out common stop words and keep only significant terms
    let stop_words = [
        // Common English stop words
        "the", "and", "for", "with", "from", "this", "that", "what", "how",
        "are", "was", "were", "been", "being", "have", "has", "had", "does",
        "did", "will", "would", "could", "should", "may", "might", "must",
        "can", "about", "into", "through", "during", "before", "after",
        "above", "below", "between", "under", "again", "further", "then",
        "once", "here", "there", "when", "where", "why", "all", "any",
        "both", "each", "few", "more", "most", "other", "some", "such",
        "only", "own", "same", "than", "too", "very", "just", "but",
        // Conversational filler words
        "hey", "hello", "hi", "please", "thanks", "thank", "you", "your",
        "want", "need", "help", "tell", "show", "give", "get", "make",
        "called", "named", "like", "know", "think", "see", "look",
        // Question words
        "who", "whom", "which", "whose",
        // Common verbs that add little meaning
        "doing", "done", "going", "gone", "come", "came",
    ];
    let keywords: Vec<String> = query
        .split_whitespace()
        .filter(|w| w.len() > 2) // Skip very short words
        .filter(|w| !stop_words.contains(&w.to_lowercase().as_str())) // Skip stop words
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string()) // Remove punctuation
        .filter(|w| !w.is_empty()) // Remove empty strings after trimming
        .collect();
    
    println!("  Extracted keywords: {:?}", keywords);
    
    // Expand keywords for better coverage
    let expanded_keywords = expand_query_keywords(&keywords);
    println!("  Expanded to: {:?}", expanded_keywords);
    
    // Strategy 2: BM25 Full-Text Search with expanded keywords
    let mut keyword_count = 0;
    if !expanded_keywords.is_empty() {
        if let Ok(keyword_messages) = search_messages_by_keywords(client, &expanded_keywords, top_k * 3).await {
            keyword_count = keyword_messages.len();
            println!("  BM25 search found {} messages", keyword_count);
            
            // Calculate keyword coverage for relevance filtering
            for msg in keyword_messages {
                if message_ids.insert(msg.message_id) {
                    let content_lower = msg.content.to_lowercase();
                    let mut original_keyword_matches = 0;
                    
                    // Check coverage of ORIGINAL keywords (not expanded)
                    // Prioritize rare/specific keywords (longer = more specific)
                    let mut weighted_matches = 0.0;
                    let mut total_weight = 0.0;
                    let mut has_longest_keyword = false;
                    
                    // Find the longest keyword (most specific)
                    let longest_keyword = keywords.iter()
                        .max_by_key(|k| k.len())
                        .map(|k| k.to_lowercase());
                    
                    for kw in &keywords {
                        // Weight by length: longer keywords are more specific
                        let weight = (kw.len() as f32).max(1.0);
                        total_weight += weight;
                        
                        if content_lower.contains(&kw.to_lowercase()) {
                            weighted_matches += weight;
                            
                            // Check if this is the longest keyword
                            if Some(kw.to_lowercase()) == longest_keyword {
                                has_longest_keyword = true;
                            }
                        }
                    }
                    
                    let coverage = if total_weight > 0.0 { 
                        weighted_matches / total_weight
                    } else { 
                        0.0 
                    };
                    
                    // STRICT: Must contain the longest (most specific) keyword
                    // OR have decent BM25 score (>0.01) with good coverage (>50%)
                    // BUT: If longest keyword is very specific (>8 chars), it MUST be present
                    let longest_keyword_len = longest_keyword.as_ref().map(|k| k.len()).unwrap_or(0);
                    let require_longest = longest_keyword_len > 8; // Specific terms like "editdistance" (13 chars)
                    
                    if has_longest_keyword || (!require_longest && (msg.relevance_score > 0.01 && coverage >= 0.5)) {
                        let mut boosted_msg = msg;
                        // Boost based on coverage: 20% = 1.5x, 100% = 3.0x
                        // Higher boost for better coverage: 40% = 2.0x, 100% = 4.0x
                        let boost = 2.0 + (coverage * 2.0);
                        boosted_msg.relevance_score *= boost;
                        println!("    ✓ Message (weighted coverage: {:.0}%, boost: {:.1}x)", coverage * 100.0, boost);
                        results.push(boosted_msg);
                    } else {
                        println!("    ⚠️  Filtered out (weighted coverage: {:.0}%, score: {:.2})", coverage * 100.0, msg.relevance_score);
                    }
                }
            }
        }
    }
    // This prevents poor-quality embeddings from polluting good keyword results
    if keyword_count < (top_k as usize) {
        let remaining = top_k - (keyword_count as i64);
        if let Ok(embedding_messages) = get_similar_messages_by_embedding(client, query_embedding, remaining).await {
            println!("  Embedding search found {} additional messages", embedding_messages.len());
            for msg in embedding_messages {
                if message_ids.insert(msg.message_id) {
                    // Downweight embedding scores to prioritize keyword matches
                    let mut adjusted_msg = msg;
                    adjusted_msg.relevance_score *= 0.8; // 20% penalty for embedding-only matches
                    results.push(adjusted_msg);
                }
            }
        }
    } else {
        println!("  Skipping embedding search (keyword search found enough results)");
    }
    
    // Sort by relevance score (keyword matches first, then by embedding similarity)
    results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
    
    // Limit to top_k
    results.truncate(top_k as usize);
    
    Ok(results)
}

