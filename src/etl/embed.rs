use anyhow::Result;
use serde_json::json;
use std::time::Instant;

/// Generate embedding for text using llama.cpp HTTP server
/// Falls back to placeholder if server is not configured
pub async fn embed_text(text: &str) -> Result<Vec<f32>> {
    let start = Instant::now();
    let cfg = crate::config::Config::from_env();
    
    eprintln!("🔍 embed_text called with text length: {} chars", text.len());
    eprintln!("   First 50 chars: {}", &text.chars().take(50).collect::<String>());
    
    // Try to use HTTP server first
    if let Some(server_url) = cfg.embed_server_url {
        eprintln!("🌐 Attempting HTTP embedding via: {}", server_url);
        match embed_via_http(&server_url, text).await {
            Ok(embedding) => {
                let duration = start.elapsed();
                eprintln!("✅ HTTP embedding successful:");
                eprintln!("   Dimension: {}", embedding.len());
                eprintln!("   Duration: {:?}", duration);
                eprintln!("   First 5 values: {:?}", &embedding[..5.min(embedding.len())]);
                return Ok(embedding);
            }
            Err(e) => {
                eprintln!("❌ HTTP embedding failed: {}", e);
                eprintln!("   Error details: {:?}", e);
                eprintln!("   Falling back to placeholder embeddings");
            }
        }
    } else {
        eprintln!("⚠️  EMBED_SERVER_URL not set in .env");
        eprintln!("   Add: EMBED_SERVER_URL=http://localhost:8080");
    }
    
    // Fallback to placeholder
    eprintln!("⚠️  Using placeholder embeddings (all 0.1)");
    Ok(vec![0.1f32; 768])
}

async fn embed_via_http(server_url: &str, text: &str) -> Result<Vec<f32>> {
    let start = Instant::now();
    
    eprintln!("   → Building HTTP client...");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| {
            eprintln!("   ❌ Failed to build HTTP client: {}", e);
            e
        })?;
    
    let endpoint = format!("{}/embedding", server_url);
    eprintln!("   → Sending POST request to: {}", endpoint);
    
    let payload = json!({ "content": text });
    eprintln!("   → Payload: {}", serde_json::to_string(&payload).unwrap_or_default());
    
    let response = client
        .post(&endpoint)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            eprintln!("   ❌ HTTP request failed: {}", e);
            eprintln!("      Is the llama.cpp server running on {}?", server_url);
            e
        })?;
    
    let status = response.status();
    eprintln!("   → Response status: {}", status);
    
    if !status.is_success() {
        let body = response.text().await?;
        eprintln!("   ❌ Server returned error body: {}", body);
        return Err(anyhow::anyhow!(
            "Embedding server returned {}: {}", 
            status, 
            body
        ));
    }
    
    eprintln!("   → Parsing JSON response...");
    let result: serde_json::Value = response.json().await.map_err(|e| {
        eprintln!("   ❌ Failed to parse JSON: {}", e);
        e
    })?;
    
    // llama.cpp server returns: [{"index": 0, "embedding": [[...values...]]}]
    // We need to extract the first item's embedding array
    eprintln!("   → Extracting embedding from response...");
    
    let embedding = if let Some(arr) = result.as_array() {
        // Response is an array, get first item
        eprintln!("   → Response is array with {} items", arr.len());
        arr.get(0)
            .and_then(|item| item["embedding"].as_array())
            .and_then(|emb_arr| emb_arr.get(0))
            .and_then(|inner| inner.as_array())
            .ok_or_else(|| {
                eprintln!("   ❌ Unexpected array structure");
                eprintln!("      Response: {}", serde_json::to_string(&result).unwrap_or_default());
                anyhow::anyhow!("Unexpected array structure in response")
            })?
    } else if let Some(obj) = result.as_object() {
        // Response is an object, try direct embedding field
        eprintln!("   → Response is object with keys: {:?}", obj.keys().collect::<Vec<_>>());
        result["embedding"]
            .as_array()
            .ok_or_else(|| {
                eprintln!("   ❌ No 'embedding' field in response");
                eprintln!("      Response: {}", serde_json::to_string(&result).unwrap_or_default());
                anyhow::anyhow!("No 'embedding' field in response")
            })?
    } else {
        eprintln!("   ❌ Response is neither array nor object");
        return Err(anyhow::anyhow!("Invalid response format"));
    };
    
    let embedding: Vec<f32> = embedding
        .iter()
        .map(|v| v.as_f64().ok_or_else(|| anyhow::anyhow!("Invalid embedding value")))
        .collect::<Result<Vec<f64>>>()?
        .into_iter()
        .map(|f| f as f32)
        .collect();
    
    let duration = start.elapsed();
    eprintln!("   ✅ Embedding extracted successfully in {:?}", duration);
    
    Ok(embedding)
}
