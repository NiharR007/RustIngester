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
    pub retrieval_mode: Option<String>, // "hybrid" (default), "kg_only", "direct_only"
}

#[derive(Debug, Serialize)]
pub struct ContextQueryResponse {
    pub formatted_context: FormattedLLMContext,
    pub knowledge_graph_edges: Vec<KGEdgeWithContext>,
    pub query_duration_ms: u128,
    pub total_evidence_messages: usize,
    pub retrieval_stats: RetrievalStats,
}

#[derive(Debug, Serialize)]
pub struct RetrievalStats {
    pub kg_edge_matches: usize,
    pub direct_message_matches: usize,
    pub total_unique_messages: usize,
    pub retrieval_mode: String,
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
    let include_kg_edges = payload.include_kg_edges.unwrap_or(false);
    let retrieval_mode = payload.retrieval_mode.as_deref().unwrap_or("hybrid");

    println!("Retrieval mode: {}", retrieval_mode);

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

    // Step 2A: Search KG edges with graph traversal (if enabled)
    let mut kg_edge_count = 0;
    let mut evidence_message_ids = HashSet::new();
    let mut kg_edges_for_response = Vec::new();

    if retrieval_mode == "hybrid" || retrieval_mode == "kg_only" {
        // Use hybrid KG retrieval with graph traversal
        let enable_traversal = true; // Enable multi-hop traversal
        let kg_edges = match hybrid_kg_retrieval(&client, &query_embedding, top_k as i64, enable_traversal).await {
            Ok(edges) => edges,
            Err(e) => {
                eprintln!("Error in hybrid KG retrieval: {}", e);
                if retrieval_mode == "kg_only" {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                Vec::new() // Continue with direct search in hybrid mode
            }
        };

        kg_edge_count = kg_edges.len();
        println!("Found {} edges via KG search + graph traversal", kg_edge_count);

        // Extract evidence_message_ids from matched edges with relevance filtering
        for edge in kg_edges {
            println!("  KG Edge: {} {} {}", 
                edge.source, edge.relation, edge.target);
            
            // Check if edge is relevant to query keywords
            let edge_text = format!("{} {} {}", edge.source, edge.relation, edge.target).to_lowercase();
            let query_lower = payload.query.to_lowercase();
            
            // Simple relevance check: edge contains at least one query word (>3 chars)
            let query_words: Vec<&str> = query_lower.split_whitespace()
                .filter(|w| w.len() > 3)
                .collect();
            
            let is_relevant = query_words.iter().any(|word| edge_text.contains(word));
            
            if is_relevant || retrieval_mode == "kg_only" {
                for msg_id in &edge.evidence_message_ids {
                    evidence_message_ids.insert(*msg_id);
                }
                kg_edges_for_response.push(edge);
            } else {
                println!("    ⚠️  Filtered out (not relevant to query)");
            }
        }

        println!("Collected {} unique message IDs from KG (with traversal)", evidence_message_ids.len());
    }

    // Step 2B: HYBRID/DIRECT - Search messages with keyword + embedding hybrid
    let mut direct_message_count = 0;
    if retrieval_mode == "hybrid" || retrieval_mode == "direct_only" {
        println!("Using hybrid keyword + embedding search for direct messages");
        
        let similar_messages = match hybrid_search_messages(&client, &payload.query, &query_embedding, top_k as i64).await {
            Ok(msgs) => msgs,
            Err(e) => {
                eprintln!("Error in hybrid message search: {}", e);
                if retrieval_mode == "direct_only" {
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                Vec::new() // Continue with KG results in hybrid mode
            }
        };

        direct_message_count = similar_messages.len();
        println!("Found {} messages via hybrid search (keyword + embedding)", direct_message_count);

        // Add directly matched messages to the evidence set
        for msg_with_rel in &similar_messages {
            let preview = if msg_with_rel.content.len() > 60 {
                &msg_with_rel.content[..60]
            } else {
                &msg_with_rel.content
            };
            println!("  Match: {}... (score: {:.3})", 
                preview, msg_with_rel.relevance_score);
            evidence_message_ids.insert(msg_with_rel.message_id);
        }

        println!("Total unique message IDs after hybrid search: {}", evidence_message_ids.len());
    }

    // Step 3: Fetch the actual messages using the combined evidence_message_ids
    let evidence_message_vec: Vec<Uuid> = evidence_message_ids.into_iter().collect();
    let messages = match get_messages_by_ids_ordered(&client, &evidence_message_vec).await {
        Ok(msgs) => msgs,
        Err(e) => {
            eprintln!("Error fetching messages: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    println!("Retrieved {} messages (KG: {}, Direct: {}, Mode: {})", 
        messages.len(), kg_edge_count, direct_message_count, retrieval_mode);

    let total_evidence_messages = messages.len();

    // Step 4: Format messages for LLM context with token management
    let formatted = format_messages_for_llm_simple(messages, max_tokens);

    println!("Formatted {} messages for LLM (estimated {} tokens, {:.1}% of context window)",
        formatted.messages.len(),
        formatted.total_tokens_estimate,
        formatted.context_window_used);

    let response = ContextQueryResponse {
        formatted_context: formatted,
        knowledge_graph_edges: if include_kg_edges { kg_edges_for_response } else { Vec::new() },
        query_duration_ms: start.elapsed().as_millis(),
        total_evidence_messages,
        retrieval_stats: RetrievalStats {
            kg_edge_matches: kg_edge_count,
            direct_message_matches: direct_message_count,
            total_unique_messages: total_evidence_messages,
            retrieval_mode: retrieval_mode.to_string(),
        },
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

/// Format messages for LLM (simple version without relevance scores)
/// Used when messages come from evidence_message_ids (already relevant by definition)
fn format_messages_for_llm_simple(
    messages: Vec<Message>,
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
            relevance_score: 1.0, // All evidence messages are equally relevant
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

