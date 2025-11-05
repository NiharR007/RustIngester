# RustIngester Docker Setup

Run RustIngester with a single command using Docker Compose!

## Prerequisites

- **Docker Desktop** (Mac/Windows) or **Docker Engine** (Linux)
- **Docker Compose** v2.0 or higher
- **4GB RAM** minimum (8GB recommended)
- **5GB disk space** for images and models

## Quick Start (3 Steps)

### 1. Download the Model (One-time)

```bash
./download-model.sh
```

This downloads the Nomic Embed model (~74MB) to the `models/` directory.

### 2. Start All Services

```bash
./docker-start.sh
```

This will:
- Build the RustIngester Docker image
- Start PostgreSQL with pgvector
- Start llama.cpp embedding server
- Start RustIngester API service

### 3. Test the Service

```bash
# Check status
curl http://localhost:3000/status | jq .

# Get statistics
curl http://localhost:3000/ingest/statistics | jq .
```

## Services

The Docker Compose setup runs 3 containers:

| Service | Port | Description |
|---------|------|-------------|
| **PostgreSQL** | 5432 | Database with pgvector extension |
| **llama-server** | 8080 | Embedding generation service |
| **rustingester** | 3000 | Main API service |

## Usage

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f rustingester
docker-compose logs -f llama-server
docker-compose logs -f postgres
```

### Stop Services

```bash
./docker-stop.sh
```

Or manually:
```bash
docker-compose down
```

### Remove All Data (Including Database)

```bash
docker-compose down -v
```

⚠️ **Warning**: This deletes all ingested data!

### Restart Services

```bash
docker-compose restart
```

### Rebuild After Code Changes

```bash
docker-compose up -d --build rustingester
```

## Ingesting Data

Once the services are running, ingest your data:

```bash
# Ingest messages with embeddings
curl -X POST http://localhost:3000/ingest/messages \
  -H "Content-Type: application/json" \
  -d @Data/turn_embeddings.json

# Ingest knowledge graph
curl -X POST http://localhost:3000/ingest/knowledge-graph \
  -H "Content-Type: application/json" \
  -d @Data/enhanced_pipeline_full_results.json
```

## Querying

```bash
# Query for LLM context
curl -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How do I install a Python package?",
    "top_k": 5,
    "max_tokens": 2000
  }' | jq .
```

## Troubleshooting

### "Model file not found"

Download the model first:
```bash
./download-model.sh
```

### "Docker is not running"

Start Docker Desktop or Docker Engine.

### "Port already in use"

Stop conflicting services:
```bash
# Stop local PostgreSQL
brew services stop postgresql@14  # macOS
sudo systemctl stop postgresql    # Linux

# Or change ports in docker-compose.yml
```

### "Container won't start"

Check logs:
```bash
docker-compose logs rustingester
```

Common issues:
- Database not ready: Wait 10-15 seconds for PostgreSQL to initialize
- Model missing: Run `./download-model.sh`
- Out of memory: Increase Docker memory limit in Docker Desktop

### Reset Everything

```bash
docker-compose down -v
docker system prune -f
./docker-start.sh
```

## Configuration

Edit `docker-compose.yml` to customize:

```yaml
environment:
  DATABASE_URL: postgresql://postgres:postgres@postgres:5432/rustingester
  EMBED_SERVER_URL: http://llama-server:8080
  LSH_BUCKETS: 8
  SERVER_PORT: 3000
  RUST_LOG: info  # Change to 'debug' for verbose logs
```

## Production Deployment

For production use:

1. **Change default passwords** in `docker-compose.yml`
2. **Use environment variables** for secrets
3. **Add volume mounts** for persistent data
4. **Configure reverse proxy** (nginx/traefik)
5. **Enable SSL/TLS** for API endpoints
6. **Set resource limits**:

```yaml
services:
  rustingester:
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 4G
        reservations:
          memory: 2G
```

## Architecture

```
┌─────────────────────────────────────────┐
│          Docker Compose Network         │
│                                         │
│  ┌──────────────┐   ┌──────────────┐  │
│  │  PostgreSQL  │   │ llama-server │  │
│  │  (pgvector)  │   │  (embeddings)│  │
│  │  :5432       │   │  :8080       │  │
│  └──────┬───────┘   └──────┬───────┘  │
│         │                  │           │
│         └────────┬─────────┘           │
│                  │                     │
│         ┌────────▼──────────┐          │
│         │  RustIngester     │          │
│         │     :3000         │          │
│         └───────────────────┘          │
│                  │                     │
└──────────────────┼─────────────────────┘
                   │
              [Your Client]
         http://localhost:3000
```

## Support

For issues with Docker setup:
- Check logs: `docker-compose logs`
- Verify disk space: `docker system df`
- Check Docker version: `docker --version` (20.10+ required)

---

**Built with ❤️ using Docker, Rust, and PostgreSQL**
