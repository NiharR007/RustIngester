# RustIngester - Docker Quick Start

Get RustIngester running in **under 2 minutes** with Docker!

## Prerequisites

- Docker Desktop (Mac/Windows) or Docker Engine (Linux)
- 4GB RAM minimum
- 5GB disk space

## Quick Start

### 1. Start Everything

```bash
# Download model (one-time, ~74MB)
./download-model.sh

# Start all services
docker compose up -d
```

That's it! The service will be available at `http://localhost:3000`

### 2. Verify It's Running

```bash
# Check status
curl http://localhost:3000/status | jq .

# Expected output:
# {
#   "status": "healthy",
#   "database": "connected",
#   "age_extension": "loaded",
#   "total_nodes": 0,
#   "total_edges": 0
# }
```

### 3. Ingest Your Data

```bash
# Ingest messages with embeddings (if you have data)
curl -X POST http://localhost:3000/ingest/messages \
  -H "Content-Type: application/json" \
  -d @Data/turn_embeddings.json

# Ingest knowledge graph (if you have data)
curl -X POST http://localhost:3000/ingest/knowledge-graph \
  -H "Content-Type: application/json" \
  -d @Data/enhanced_pipeline_full_results.json
```

### 4. Query with Semantic Search

```bash
curl -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How do I install a Python package?",
    "top_k": 5,
    "max_tokens": 2000
  }' | jq .
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
