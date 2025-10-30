# Quick Start Guide

## 1. Build the Project

```bash
cd /Users/niharpatel/Desktop/RustIngester
cargo build --release
```

## 2. Test with CLI (Fastest Way)

```bash
# Ingest the ok.json file directly
cargo run --release --bin ingest_cli -- Data/ok.json
```

## 3. Start the Web Service

```bash
# Terminal 1: Start the service
cargo run --release --bin service
```

```bash
# Terminal 2: Test the API
curl http://localhost:3000/status | jq

# Ingest the data
curl -X POST http://localhost:3000/ingest/batch \
  -H "Content-Type: application/json" \
  -d @Data/ok.json | jq

# Query similar edges
curl -X POST http://localhost:3000/query/similar \
  -H "Content-Type: application/json" \
  -d '{
    "query": "user wants to install python package",
    "top_k": 5
  }' | jq
```

## 4. Verify in Database

```bash
psql -U postgres
```

```sql
SET search_path = ag_catalog, "$user", public;

-- Check sessions
SELECT * FROM sessions;

-- Check embeddings
SELECT session_id, edge_text FROM embeddings LIMIT 10;

-- Query graph nodes
SELECT * FROM ag_catalog.cypher('sem_graph', $$
  MATCH (n) RETURN n LIMIT 10
$$) AS (result agtype);
```

## Available Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/status` | GET | Health check |
| `/ingest/session` | POST | Ingest single session |
| `/ingest/batch` | POST | Ingest multiple sessions |
| `/query/similar` | POST | Query similar edges |
| `/query/session/:id` | GET | Get session graph |
| `/graph/cypher` | POST | Execute Cypher query |

## Files Created

- ✅ `IMPLEMENTATION_GUIDE.md` - Complete technical documentation
- ✅ `TESTING_GUIDE.md` - Detailed testing instructions
- ✅ `src/api/` - Web service API module
- ✅ `src/bin/service.rs` - Web service binary
- ✅ `src/bin/ingest_cli.rs` - CLI ingestion tool
- ✅ Updated `src/etl/parser.rs` - ok.json models
- ✅ Updated `src/ingest.rs` - Batch ingestion functions
- ✅ Updated `src/db/` - Evidence tracking support

## Next Steps

1. Build the project: `cargo build --release`
2. Test CLI ingestion: `cargo run --release --bin ingest_cli -- Data/ok.json`
3. Start service: `cargo run --release --bin service`
4. Test API endpoints (see TESTING_GUIDE.md)
5. Verify data in PostgreSQL
6. Scale test with larger datasets
