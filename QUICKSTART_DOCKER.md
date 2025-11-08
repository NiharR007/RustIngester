# RustIngester - Docker Quick Start

Get RustIngester running in **under 5 minutes** with Docker! Includes PostgreSQL with **Apache AGE** and **pgvector**.

## Prerequisites

- Docker Desktop (Mac/Windows) or Docker Engine (Linux)
- 4GB RAM minimum
- 10GB disk space (includes PostgreSQL build with AGE)
- `curl` and `jq` for testing

## Quick Start

### 1. Download the Embedding Model

```bash
# Download Nomic Embed model (one-time, ~74MB)
./download-model.sh
```

### 2. Build and Start All Services

```bash
# Build custom PostgreSQL image with AGE (first time: ~5 minutes)
# Subsequent starts: ~10 seconds
docker compose up -d --build
```

**What's happening:**
- Building PostgreSQL 14 with pgvector 0.8.0 and Apache AGE 1.5.0
- Starting llama.cpp embedding server
- Starting RustIngester API service

### 3. Wait for Services to be Ready

```bash
# Wait for all services to be healthy (~30 seconds)
echo "Waiting for services..."
sleep 30

# Check status
curl http://localhost:3000/status | jq .
```

**Expected output:**
```json
{
  "status": "healthy",
  "database": "connected",
  "age_extension": "loaded",
  "graph_name": "sem_graph",
  "total_nodes": 0,
  "total_edges": 0
}
```

That's it! The service is now running at `http://localhost:3000`

### 4. Test Hybrid Retrieval (Optional - if you have data)

If you have the sample data files, you can test the full system:

#### Ingest Messages

```bash
# Ingest messages with embeddings
curl -X POST http://localhost:3000/ingest/messages \
  -H "Content-Type: application/json" \
  -d @Data/turn_embeddings.json | jq .

# Expected: {"success": true, "total_processed": 5741, ...}
```

#### Ingest Knowledge Graph

```bash
# Ingest KG edges (takes ~25 seconds for embedding generation)
curl -X POST http://localhost:3000/ingest/knowledge-graph \
  -H "Content-Type: application/json" \
  -d @Data/enhanced_pipeline_full_results.json | jq .

# Expected: {"success": true, "total_processed": 3329, ...}
```

#### Test Hybrid Retrieval

```bash
# Test BM25 keyword search (direct_only mode)
curl -s -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Zapier",
    "top_k": 5,
    "retrieval_mode": "direct_only"
  }' | jq '{
    stats: .retrieval_stats,
    first_message: .formatted_context.messages[0].content[:100]
  }'

# Expected: 5 messages about Zapier with 100% keyword coverage
```

#### Test Query Expansion

```bash
# Test with technical term
curl -s -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "install package",
    "top_k": 5,
    "retrieval_mode": "direct_only"
  }' | jq '.retrieval_stats'

# Query expands "install" → ["install", "setup", "pip", "npm", ...]
```

## Common Commands

```bash
# View logs
docker compose logs -f rustingester

# Stop services
docker compose down

# Restart services
docker compose restart

# Remove all data (including database)
docker compose down -v
```

## What's Running

| Service | Port | Purpose |
|---------|------|---------|
| **RustIngester API** | 3000 | Main RAG service |
| **llama.cpp** | 8080 | Embedding generation |
| **PostgreSQL** | 5432 | Database with pgvector |

## Simplified Installation vs Manual

### Docker (Recommended)
```bash
./download-model.sh  # One-time
docker compose up -d # Start everything
```
**Time: 2 minutes**

### Manual Installation
1. Install PostgreSQL 14
2. Install pgvector extension
3. Install Apache AGE extension
4. Clone and build llama.cpp
5. Download model
6. Configure .env
7. Build Rust project
8. Start 3 separate services

**Time: 30+ minutes**

## Troubleshooting

### "Connection refused"
```bash
# Wait a few seconds for services to start
sleep 10 && curl http://localhost:3000/status
```

### "Port already in use"
```bash
# Stop local services
brew services stop postgresql@14  # macOS
pkill -f llama-server

# Or change ports in docker-compose.yml
```

### View detailed logs
```bash
docker compose logs rustingester
docker compose logs postgres
docker compose logs llama-server
```

### Reset everything
```bash
docker compose down -v
docker system prune -f
docker compose up -d
```

## Production Deployment

For production, update `docker-compose.yml`:

1. **Change passwords**:
```yaml
environment:
  POSTGRES_PASSWORD: <your-secure-password>
```

2. **Add resource limits**:
```yaml
deploy:
  resources:
    limits:
      memory: 4G
```

3. **Use volumes for persistence**:
```yaml
volumes:
  - ./postgres-data:/var/lib/postgresql/data
```

## Architecture

```
┌─────────────────────────────────────┐
│      Docker Compose Network         │
│                                     │
│  ┌────────────┐  ┌──────────────┐  │
│  │ PostgreSQL │  │ llama-server │  │
│  │ (pgvector) │  │ (embeddings) │  │
│  └──────┬─────┘  └──────┬───────┘  │
│         │                │          │
│    ┌────┴────────────────┴────┐    │
│    │    RustIngester :3000    │    │
│    └──────────────────────────┘    │
└─────────────────────────────────────┘
                 │
         [Your Application]
      http://localhost:3000
```

## Next Steps

- Check out [DOCKER.md](DOCKER.md) for detailed documentation
- See [README.md](README.md) for API reference
- View [TESTING_GUIDE.md](TESTING_GUIDE.md) for testing instructions

---

**Your RAG system is now running!** 🚀
