use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;
use crate::db::{models::*, message_ops::*, kg_ops::*, connect::get_client};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ContextQueryRequest {
    pub query: String,
    pub top_k: Option<usize>,
    pub max_tokens: Option<usize>, // e.g., 4000 for context window
    pub include_kg_edges: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ContextQueryResponse {
    pub formatted_context: FormattedLLMContext,
    pub knowledge_graph_edges: Vec<KGEdgeWithContext>,
    pub query_duration_ms: u128,
    pub total_evidence_messages: usize,
}

// ============================================================================
// LLM Context Query Handler
// ============================================================================

/// Query for LLM context based on a natural language query
/// This retrieves relevant knowledge graph edges and their associated message content
pub async fn query_llm_context(
    Json(payload): Json<ContextQueryRequest>,
) -> Result<Json<ContextQueryResponse>, StatusCode> {
    let start = std::time::Instant::now();

    let top_k = payload.top_k.unwrap_or(10);
    let max_tokens = payload.max_tokens.unwrap_or(4000);
    let include_kg_edges = payload.include_kg_edges.unwrap_or(true);

    println!("Querying LLM context for: '{}' (top_k={}, max_tokens={})",
        payload.query, top_k, max_tokens);

    let client = match get_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Step 1: Generate embedding for the query using llama.cpp server
    use crate::etl::embed;
    let query_embedding = match embed::embed_text(&payload.query).await {
        Ok(emb) => emb,
        Err(e) => {
            eprintln!("Error generating query embedding: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    println!("Generated query embedding with {} dimensions", query_embedding.len());

    // Step 2: Perform semantic similarity search on message embeddings
    let similar_messages = match get_similar_messages_by_embedding(&client, &query_embedding, top_k as i64 * 2).await {
        Ok(msgs) => msgs,
        Err(e) => {
            eprintln!("Error querying similar messages: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    println!("Found {} similar messages via embedding search", similar_messages.len());

    // Step 3: Extract keywords for KG search (parallel with semantic search)
    let keywords: Vec<String> = payload.query
        .split_whitespace()
        .filter(|s| s.len() > 2)
        .map(|s| s.to_lowercase())
        .collect();

    // Step 4: Query knowledge graph for relevant edges (for additional context)
    let kg_edges = match get_edges_by_query(&client, &keywords, top_k as i64).await {
        Ok(edges) => edges,
        Err(e) => {
            eprintln!("Error querying knowledge graph: {}", e);
            Vec::new() // Don't fail, just return empty
        }
    };

    println!("Found {} relevant KG edges", kg_edges.len());

    let total_evidence_messages = similar_messages.len();

    // Step 5: Format for LLM context with token estimation and actual relevance scores
    let formatted = format_messages_with_scores(similar_messages, max_tokens);

    println!("Formatted {} messages for LLM (estimated {} tokens, {:.1}% of context window)",
        formatted.messages.len(),
        formatted.total_tokens_estimate,
        formatted.context_window_used);

    let response = ContextQueryResponse {
        formatted_context: formatted,
        knowledge_graph_edges: if include_kg_edges { kg_edges } else { Vec::new() },
        query_duration_ms: start.elapsed().as_millis(),
        total_evidence_messages,
    };

    Ok(Json(response))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Format messages with actual relevance scores from embedding similarity
fn format_messages_with_scores(
    messages: Vec<MessageWithRelevance>,
    max_tokens: usize,
) -> FormattedLLMContext {
    let mut llm_messages = Vec::new();
    let mut total_tokens = 0;
    let tokens_per_char = 0.25; // rough estimate: 1 token ≈ 4 chars

    // Track unique conversations
    let mut conversations = HashSet::new();

    for msg in messages.iter() {
        let estimated_tokens = (msg.content.len() as f32 * tokens_per_char) as usize;

        // Stop if we exceed token budget
        if total_tokens + estimated_tokens > max_tokens {
            println!("Reached token limit, stopping at {} messages", llm_messages.len());
            break;
        }

        conversations.insert(msg.conversation_id);

        // Parse role from message content if possible
        let (role, content) = parse_message_role(&msg.content);

        llm_messages.push(LLMContextMessage {
            role,
            content,
            message_id: msg.message_id,
            relevance_score: msg.relevance_score,
        });

        total_tokens += estimated_tokens;
    }

    FormattedLLMContext {
        messages: llm_messages,
        total_tokens_estimate: total_tokens,
        context_window_used: (total_tokens as f32 / max_tokens as f32) * 100.0,
        unique_conversations: conversations.len(),
    }
}

/// Format messages for LLM consumption with token budget management (legacy version)
#[allow(dead_code)]
fn format_messages_for_llm(
    messages: Vec<Message>,
    max_tokens: usize,
) -> FormattedLLMContext {
    let mut llm_messages = Vec::new();
    let mut total_tokens = 0;
    let tokens_per_char = 0.25; // rough estimate: 1 token ≈ 4 chars

    // Track unique conversations
    let mut conversations = HashSet::new();

    for (idx, msg) in messages.iter().enumerate() {
        let estimated_tokens = (msg.content.len() as f32 * tokens_per_char) as usize;

        // Stop if we exceed token budget
        if total_tokens + estimated_tokens > max_tokens {
            println!("Reached token limit at message {} of {}", idx + 1, messages.len());
            break;
        }

        conversations.insert(msg.conversation_id);

        // Calculate relevance score (higher for earlier messages in result set)
        let relevance_score = 1.0 - (idx as f32 / messages.len().max(1) as f32);

        // Parse role from message content if possible
        let (role, content) = parse_message_role(&msg.content);

        llm_messages.push(LLMContextMessage {
            role,
            content,
            message_id: msg.message_id,
            relevance_score,
        });

        total_tokens += estimated_tokens;
    }

    FormattedLLMContext {
        messages: llm_messages,
        total_tokens_estimate: total_tokens,
        context_window_used: (total_tokens as f32 / max_tokens as f32) * 100.0,
        unique_conversations: conversations.len(),
    }
}

/// Parse message role from content (e.g., "user: hello" -> ("user", "hello"))
fn parse_message_role(content: &str) -> (String, String) {
    // Check if content starts with a role prefix like "user:" or "assistant:"
    let parts: Vec<&str> = content.splitn(2, ':').collect();

    if parts.len() == 2 {
        let potential_role = parts[0].trim().to_lowercase();
        if matches!(potential_role.as_str(), "user" | "assistant" | "system") {
            return (potential_role, parts[1].trim().to_string());
        }
    }

    // Default to "user" if no role prefix found
    ("user".to_string(), content.to_string())
}

// ============================================================================
// Direct Message Query Handler
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct MessageQueryRequest {
    pub message_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MessageQueryResponse {
    pub messages: Vec<Message>,
    pub total_found: usize,
}

/// Get full messages by their IDs
pub async fn query_messages_by_ids(
    Json(payload): Json<MessageQueryRequest>,
) -> Result<Json<MessageQueryResponse>, StatusCode> {
    println!("Querying {} message IDs", payload.message_ids.len());

    let client = match get_client().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to database: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    match get_messages_by_ids_ordered(&client, &payload.message_ids).await {
        Ok(messages) => {
            let total_found = messages.len();
            println!("Found {} messages", total_found);

            Ok(Json(MessageQueryResponse {
                messages,
                total_found,
            }))
        }
        Err(e) => {
            eprintln!("Error querying messages: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

