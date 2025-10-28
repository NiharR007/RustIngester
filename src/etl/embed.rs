use anyhow::Result;

/// Placeholder: call external model or OpenAI. Returns 768-dim vector filled with 0.1.
pub async fn embed_text(_text: &str) -> Result<Vec<f32>> {
    Ok(vec![0.1f32; 768])
}
