# RustIngester - Quick Reference Card

## 🚀 Quick Start (Docker)

```bash
# Start everything
./download-model.sh && docker compose up -d --build

# Check status (wait 30 seconds first)
curl http://localhost:3000/status | jq .

# Stop everything
docker compose down
```

## 📊 API Endpoints

### Health Check
```bash
curl http://localhost:3000/status | jq .
```

### Ingest Messages
```bash
curl -X POST http://localhost:3000/ingest/messages \
  -H "Content-Type: application/json" \
  -d @Data/turn_embeddings.json
```

### Ingest Knowledge Graph
```bash
curl -X POST http://localhost:3000/ingest/knowledge-graph \
  -H "Content-Type: application/json" \
  -d @Data/enhanced_pipeline_full_results.json
```

### Query (BM25 Only)
```bash
curl -s -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "your search query",
    "top_k": 5,
    "retrieval_mode": "direct_only"
  }' | jq .
```

### Query (Hybrid: BM25 + KG)
```bash
curl -s -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "your search query",
    "top_k": 5,
    "retrieval_mode": "hybrid"
  }' | jq .
```

### Get Statistics
```bash
curl http://localhost:3000/ingest/statistics | jq .
```

## 🔧 Retrieval Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `direct_only` | BM25 keyword search only | Fast, exact matches |
| `kg_only` | Knowledge graph only | Structured relationships |
| `hybrid` | BM25 + KG combined | Best overall results |

## 🎯 Query Examples

### Single Word
```bash
# Good for: Brand names, specific terms
curl -s -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{"query": "Zapier", "top_k": 5, "retrieval_mode": "direct_only"}'
```

### Technical Terms
```bash
# Automatically expands: install → [install, setup, pip, npm, ...]
curl -s -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{"query": "install package", "top_k": 5, "retrieval_mode": "direct_only"}'
```

### Complex Query
```bash
# Uses hybrid retrieval for best results
curl -s -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{"query": "python pip install dependencies", "top_k": 10, "retrieval_mode": "hybrid"}'
```

## 🐳 Docker Commands

### View Logs
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f rustingester
docker compose logs -f postgres
docker compose logs -f llama-server
```

### Restart Services
```bash
docker compose restart
docker compose restart rustingester  # Restart specific service
```

### Reset Everything
```bash
# Remove all data and rebuild
docker compose down -v
docker compose up -d --build
```

### Check Container Status
```bash
docker compose ps
docker ps
```

## 📈 Performance Tips

### Optimize for Speed
- Use `direct_only` mode for simple queries
- Set lower `top_k` values (3-5)
- Cache frequently accessed queries

### Optimize for Accuracy
- Use `hybrid` mode for complex queries
- Set higher `top_k` values (10-20)
- Include more specific keywords

### Optimize for Recall
- Use technical terms that trigger expansion
- Use `hybrid` mode with KG traversal
- Increase `max_tokens` parameter

## 🔍 Query Expansion

### Automatically Expanded Terms
- `install` → setup, installation, pip, npm, brew
- `error` → exception, bug, issue, problem, fail
- `package` → library, module, dependency, import
- `function` → method, def, procedure, func
- `api` → endpoint, service, interface, rest
- `database` → db, storage, postgres, sql

### Not Expanded
- Proper nouns (Zapier, GitHub, etc.)
- Short words (<4 chars)
- Numbers and special characters

## 📊 Response Format

```json
{
  "formatted_context": {
    "messages": [
      {
        "role": "user|assistant",
        "content": "message text",
        "message_id": "uuid",
        "relevance_score": 0.95
      }
    ],
    "total_tokens_estimate": 1500,
    "context_window_used": 75.0,
    "unique_conversations": 3
  },
  "knowledge_graph_edges": [
    {
      "source": "node1",
      "relation": "relationship",
      "target": "node2",
      "evidence_message_ids": ["uuid1", "uuid2"]
    }
  ],
  "retrieval_stats": {
    "kg_edge_matches": 10,
    "direct_message_matches": 5,
    "total_unique_messages": 12,
    "retrieval_mode": "hybrid"
  },
  "query_duration_ms": 150
}
```

## 🛠️ Troubleshooting

### Service Won't Start
```bash
# Check logs
docker compose logs postgres

# Common fix: port conflict
docker compose down
pkill -f postgres  # Stop local postgres
docker compose up -d
```

### No Results Returned
```bash
# Check if data is loaded
curl http://localhost:3000/ingest/statistics | jq .

# Try different retrieval mode
# direct_only → hybrid → kg_only
```

### Slow Queries
```bash
# Check service health
curl http://localhost:3000/status | jq .

# Restart services
docker compose restart

# Check resource usage
docker stats
```

## 📁 Important Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | Service orchestration |
| `docker/Dockerfile.postgres` | Custom PostgreSQL with AGE |
| `docker/init-db.sql` | Database initialization |
| `src/db/message_ops.rs` | BM25 search logic |
| `src/api/context_handlers.rs` | API handlers |
| `README.md` | Full documentation |
| `QUICKSTART_DOCKER.md` | Docker quick start |

## 🔗 Useful Links

- **API**: http://localhost:3000
- **Status**: http://localhost:3000/status
- **Stats**: http://localhost:3000/ingest/statistics
- **PostgreSQL**: localhost:5432
- **llama.cpp**: http://localhost:8080

## 💡 Pro Tips

1. **Use `jq` for pretty JSON**: `curl ... | jq .`
2. **Test with small `top_k` first**: Start with 3-5, then increase
3. **Check logs when debugging**: `docker compose logs -f`
4. **Use `direct_only` for speed**: Fastest retrieval mode
5. **Use `hybrid` for accuracy**: Best overall results
6. **Restart if services hang**: `docker compose restart`

---

**Need help?** Check `README.md` or `DOCKER_DEPLOYMENT_SUCCESS.md`
