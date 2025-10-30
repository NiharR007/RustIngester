# Embedding Model Setup Guide

## Overview

RustIngester now supports **native embedding generation** using `llama.cpp` through Rust bindings. This eliminates the need for external API providers and allows you to run embeddings locally on your machine.

## Current Status

- ✅ **llama_cpp dependency added** to `Cargo.toml` with Metal acceleration (Apple Silicon)
- ✅ **Embedding function updated** in `src/etl/embed.rs` with model loading and inference
- ✅ **Graceful fallback** to placeholder embeddings if model not configured
- ✅ **Config support** for `EMBED_MODEL_PATH` environment variable

## Quick Start

### Option 1: Use Without Embeddings (Current Behavior)

The service works without a model configured - it will use placeholder embeddings:

```bash
# Just run as normal
cargo run --release --bin service
```

You'll see a warning:
```
⚠️  Using placeholder embeddings: EMBED_MODEL_PATH not set
   Set EMBED_MODEL_PATH in .env to use real embeddings
```

### Option 2: Set Up Real Embeddings

## Step 1: Download an Embedding Model

### Recommended Models

**For general-purpose embeddings:**
- **nomic-embed-text-v1.5** (Recommended)
  - Size: ~274MB (Q4_0 quantized)
  - Dimensions: 768
  - Best for: General text similarity

**For code/technical content:**
- **BAAI/bge-base-en-v1.5**
  - Size: ~223MB (Q4_0 quantized)
  - Dimensions: 768
  - Best for: Technical documentation

### Download Pre-Quantized GGUF

```bash
# Create models directory
mkdir -p /Users/niharpatel/Desktop/RustIngester/models

# Download nomic-embed-text (recommended)
cd models
curl -L -O https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_0.gguf

# Or download bge-base-en
# curl -L -O https://huggingface.co/second-state/bge-base-en-v1.5-GGUF/resolve/main/bge-base-en-v1.5-Q4_0.gguf
```

### Alternative: Convert Your Own Model

If you want a different model or quantization:

```bash
# Clone llama.cpp (for conversion tools)
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp

# Install Python dependencies
pip install -r requirements.txt

# Download a model from HuggingFace
# Example: nomic-embed-text-v1.5
git clone https://huggingface.co/nomic-ai/nomic-embed-text-v1.5

# Convert to GGUF
python convert.py nomic-embed-text-v1.5/ \
  --outfile ../RustIngester/models/nomic-embed-text-v1.5.gguf \
  --outtype q4_0
```

## Step 2: Configure the Model Path

Update your `.env` file:

```bash
# Add this line to /Users/niharpatel/Desktop/RustIngester/.env
EMBED_MODEL_PATH=models/nomic-embed-text-v1.5.Q4_0.gguf

# Or use absolute path
# EMBED_MODEL_PATH=/Users/niharpatel/Desktop/RustIngester/models/nomic-embed-text-v1.5.Q4_0.gguf
```

Complete `.env` example:
```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/postgres
LSH_BUCKETS=128
SERVER_PORT=3000
EMBED_MODEL_PATH=models/nomic-embed-text-v1.5.Q4_0.gguf
```

## Step 3: Build and Run

```bash
# Build with llama_cpp (first time will take a few minutes)
cargo build --release

# Run the service
cargo run --release --bin service
```

You should see:
```
Loading embedding model from: models/nomic-embed-text-v1.5.Q4_0.gguf
✓ Embedding model loaded successfully
🚀 RustIngester Service starting on 0.0.0.0:3000
```

## Step 4: Re-Ingest Data

To use real embeddings, re-ingest your data:

```bash
# Clear old embeddings (optional)
PGPASSWORD=postgres psql -h localhost -U postgres -d postgres -c "TRUNCATE embeddings, sessions, edge_evidence CASCADE;"

# Re-ingest with real embeddings
cargo run --release --bin ingest_cli -- Data/ok.json
```

## Performance Considerations

### Model Size vs Quality

| Quantization | Size | Quality | Speed |
|--------------|------|---------|-------|
| Q4_0 | ~274MB | Good | Fast |
| Q5_0 | ~335MB | Better | Medium |
| Q8_0 | ~535MB | Best | Slower |
| F16 | ~1.1GB | Excellent | Slowest |

**Recommendation**: Start with Q4_0 for best speed/quality balance.

### Hardware Acceleration

The `llama_cpp` dependency is configured with **Metal** support for Apple Silicon:

```toml
llama_cpp = { version = "0.3.2", default-features = false, features = ["metal"] }
```

For other platforms:
- **CUDA** (NVIDIA): Change feature to `["cuda"]`
- **CPU only**: Remove features or use `["native"]`

### Memory Usage

- **Model loading**: ~300MB-1GB depending on quantization
- **Per-request**: ~10-50MB for session context
- **Concurrent requests**: Model is shared, sessions are per-request

## Troubleshooting

### Build Errors

**Error**: `cmake not found`
```bash
brew install cmake pkg-config
```

**Error**: Metal framework not found
```bash
# Ensure Xcode Command Line Tools installed
xcode-select --install
```

**Error**: Compilation takes too long
```bash
# Use fewer CPU cores
cargo build --release -j 4
```

### Runtime Errors

**Error**: `Failed to load embedding model`
- Check file path is correct
- Verify file is a valid GGUF model
- Try absolute path instead of relative

**Error**: `Out of memory`
- Use smaller quantization (Q4_0)
- Reduce `n_ctx` in `embed.rs` (currently 512)
- Close other applications

**Warning**: `Using placeholder embeddings`
- This is normal if `EMBED_MODEL_PATH` not set
- Service will work but similarity search won't be meaningful

## Testing Embeddings

### Test CLI Ingestion

```bash
cargo run --release --bin ingest_cli -- Data/ok.json
```

Watch for:
```
Loading embedding model from: models/nomic-embed-text-v1.5.Q4_0.gguf
✓ Embedding model loaded successfully
✓ Ingested session 688e7460-...: 4 nodes, 4 edges
```

### Test Similarity Query

```bash
curl -X POST http://localhost:3000/query/similar \
  -H "Content-Type: application/json" \
  -d '{
    "query": "user wants to install python package",
    "top_k": 5
  }' | jq
```

With real embeddings, you should see:
- **Varied similarity scores** (not all 1.0)
- **Relevant results ranked higher**
- **Different distances** for different queries

## Advanced Configuration

### Custom Embedding Dimensions

Edit `src/etl/embed.rs` to change vector size:

```rust
// Change 768 to your model's dimension
let mut embedding = vec![0.0f32; 768];
```

### Batch Processing

For faster ingestion, consider batching:

```rust
// Future enhancement: batch multiple texts
pub async fn embed_batch(texts: &[String]) -> Result<Vec<Vec<f32>>> {
    // Process multiple embeddings in one model session
}
```

### Model Caching

The model is loaded once and cached globally:

```rust
static EMBED_MODEL: OnceCell<Arc<EmbedModel>> = OnceCell::const_new();
```

This means:
- ✅ Fast subsequent embeddings
- ✅ Low memory overhead
- ✅ Thread-safe sharing

## Next Steps

1. **Download a model** (nomic-embed-text-v1.5 recommended)
2. **Set EMBED_MODEL_PATH** in `.env`
3. **Rebuild** with `cargo build --release`
4. **Re-ingest** your data
5. **Test** similarity queries

## Resources

- [llama.cpp GitHub](https://github.com/ggerganov/llama.cpp)
- [llama_cpp-rs Crate](https://crates.io/crates/llama_cpp)
- [Nomic Embed Models](https://huggingface.co/nomic-ai)
- [BGE Models](https://huggingface.co/BAAI)
- [GGUF Format Spec](https://github.com/ggerganov/ggml/blob/master/docs/gguf.md)

## Support

If you encounter issues:
1. Check this guide's troubleshooting section
2. Verify your model file is valid GGUF
3. Try with placeholder embeddings first
4. Check build logs for compilation errors

---

**Status**: Ready to use! The service works with or without a model configured.
