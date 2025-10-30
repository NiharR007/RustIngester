# Testing Guide for RustIngester Service

## Prerequisites

1. **PostgreSQL with AGE** must be running
2. **Environment variables** configured in `.env`:
   ```bash
   DATABASE_URL=postgresql://postgres:password@localhost:5432/postgres
   LSH_BUCKETS=128
   SERVER_PORT=3000
   ```

## Build the Project

```bash
# Build all binaries
cargo build --release

# Or build specific binaries
cargo build --release --bin service
cargo build --release --bin ingest_cli
```

## Testing Methods

### Method 1: CLI Ingestion (Recommended for Initial Testing)

This method directly ingests the ok.json file without running the web service.

```bash
# Run the CLI ingestion tool
cargo run --release --bin ingest_cli -- Data/ok.json

# Or use the built binary
./target/release/ingest_cli Data/ok.json
```

**Expected Output:**
```
🚀 Starting ingestion from: Data/ok.json
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Ingested session 688e7460-8e78-800d-bccb-7d9d5380dc33: 4 nodes, 4 edges
✓ Ingested session 687eb3da-6d40-800d-a9f2-7fd49fdeba6c: 11 nodes, 9 edges
...

✅ Ingestion completed successfully!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Statistics:
   Total Sessions:   10
   Total Nodes:      XX
   Total Edges:      XX
   Total Embeddings: XX
   Duration:         XXX ms
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
⏱️  Total time: X.XXs
```

### Method 2: Web Service API

#### Step 1: Start the Service

```bash
# Run the service
cargo run --release --bin service

# Or use the built binary
./target/release/service
```

**Expected Output:**
```
🚀 RustIngester Service starting on 0.0.0.0:3000
📊 Endpoints:
   GET  /status
   POST /ingest/session
   POST /ingest/batch
   POST /query/similar
   GET  /query/session/:session_id
   POST /graph/cypher
✅ Server listening on 0.0.0.0:3000
```

#### Step 2: Test Endpoints

**A. Health Check**
```bash
curl http://localhost:3000/status | jq
```

Expected response:
```json
{
  "status": "healthy",
  "database": "connected",
  "age_extension": "loaded",
  "graph_name": "sem_graph",
  "total_sessions": 0,
  "total_nodes": 0,
  "total_edges": 0
}
```

**B. Ingest Batch (Entire ok.json)**
```bash
curl -X POST http://localhost:3000/ingest/batch \
  -H "Content-Type: application/json" \
  -d @Data/ok.json | jq
```

Expected response:
```json
{
  "total_sessions": 10,
  "total_nodes": 75,
  "total_edges": 65,
  "total_embeddings": 65,
  "duration_ms": 2500,
  "errors": []
}
```

**C. Ingest Single Session**
```bash
curl -X POST http://localhost:3000/ingest/session \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "test-session-001",
    "graph": {
      "nodes": [
        {"id": "Alice", "type": "Person"},
        {"id": "Python", "type": "Language"}
      ],
      "edges": [
        {
          "source": "Alice",
          "relation": "knows",
          "target": "Python",
          "evidence_message_ids": ["msg-001"]
        }
      ]
    }
  }' | jq
```

Expected response:
```json
{
  "session_id": "test-session-001",
  "nodes_created": 2,
  "edges_created": 1,
  "embeddings_created": 1,
  "duration_ms": 150
}
```

**D. Query Similar Edges**
```bash
curl -X POST http://localhost:3000/query/similar \
  -H "Content-Type: application/json" \
  -d '{
    "query": "user wants to install python package",
    "top_k": 5,
    "threshold": 0.5
  }' | jq
```

Expected response:
```json
{
  "results": [
    {
      "session_id": "688e7460-8e78-800d-bccb-7d9d5380dc33",
      "edge": {
        "source": "User",
        "relation": "requested_installation_of",
        "target": "editdistance"
      },
      "similarity": 0.92,
      "distance": 0.08,
      "evidence_message_ids": ["41389ec1-cc3e-44d5-8008-bfa94abd9954"]
    }
  ],
  "count": 1
}
```

**E. Get Session Graph**
```bash
curl http://localhost:3000/query/session/688e7460-8e78-800d-bccb-7d9d5380dc33 | jq
```

**F. Execute Cypher Query**
```bash
curl -X POST http://localhost:3000/graph/cypher \
  -H "Content-Type: application/json" \
  -d '{
    "query": "MATCH (n:Person) RETURN n LIMIT 5"
  }' | jq
```

## Verification in Database

### Check Ingested Data

```sql
-- Connect to PostgreSQL
psql -U postgres

-- Set search path for AGE
SET search_path = ag_catalog, "$user", public;

-- Check sessions
SELECT * FROM sessions;

-- Check embeddings
SELECT session_id, edge_text, lsh_bucket FROM embeddings LIMIT 10;

-- Check evidence
SELECT * FROM edge_evidence LIMIT 10;

-- Query nodes in graph
SELECT * FROM ag_catalog.cypher('sem_graph', $$
  MATCH (n) RETURN n LIMIT 10
$$) AS (result agtype);

-- Query edges with relationships
SELECT * FROM ag_catalog.cypher('sem_graph', $$
  MATCH (a)-[r]->(b) RETURN a, r, b LIMIT 10
$$) AS (a agtype, r agtype, b agtype);

-- Count nodes by type
SELECT * FROM ag_catalog.cypher('sem_graph', $$
  MATCH (n) RETURN labels(n)[0] as type, count(*) as count
$$) AS (type agtype, count agtype);
```

## Performance Testing

### Load Testing with Multiple Sessions

Create a test file with many sessions:

```bash
# Generate test data (you can create a script for this)
python3 << EOF
import json
import uuid

data = {}
for i in range(100):
    session_id = str(uuid.uuid4())
    data[session_id] = {
        "nodes": [
            {"id": f"Node_{i}_1", "type": "TestNode"},
            {"id": f"Node_{i}_2", "type": "TestNode"}
        ],
        "edges": [
            {
                "source": f"Node_{i}_1",
                "relation": "test_relation",
                "target": f"Node_{i}_2",
                "evidence_message_ids": [str(uuid.uuid4())]
            }
        ]
    }

with open("Data/test_large.json", "w") as f:
    json.dump(data, f, indent=2)
EOF

# Ingest the large file
time cargo run --release --bin ingest_cli -- Data/test_large.json
```

### Concurrent API Requests

```bash
# Install Apache Bench if not available
# brew install httpd (macOS)
# sudo apt-get install apache2-utils (Linux)

# Test concurrent requests
ab -n 100 -c 10 -H "Content-Type: application/json" \
  -p test_payload.json \
  http://localhost:3000/ingest/session
```

## Troubleshooting

### Issue: "connection refused"
**Solution:** Ensure PostgreSQL is running and DATABASE_URL is correct

```bash
# Check PostgreSQL status
pg_isready

# Test connection
psql -U postgres -c "SELECT version();"
```

### Issue: "extension 'age' does not exist"
**Solution:** Install Apache AGE extension

```bash
cd age
make PG_CONFIG=/path/to/pg_config install
psql -U postgres -c "CREATE EXTENSION age;"
```

### Issue: "label already exists"
**Solution:** This is handled automatically by the code. If persistent:

```sql
-- Drop and recreate graph (WARNING: deletes all data)
SELECT drop_graph('sem_graph', true);
SELECT create_graph('sem_graph');
```

### Issue: Slow ingestion
**Solutions:**
1. Increase LSH_BUCKETS in .env
2. Use connection pooling (future enhancement)
3. Batch smaller chunks

## Expected Performance

With the ok.json file (10 sessions, ~75 nodes, ~65 edges):
- **CLI Ingestion**: 1-3 seconds
- **API Batch Ingestion**: 2-4 seconds
- **Single Session API**: 100-300ms
- **Query Similar**: 50-200ms

## Next Steps

1. ✅ Test CLI ingestion with ok.json
2. ✅ Verify data in database
3. ✅ Start web service
4. ✅ Test all API endpoints
5. ✅ Run performance tests with larger datasets
6. 🔄 Integrate real embedding model (replace placeholder)
7. 🔄 Add authentication
8. 🔄 Add monitoring and metrics

## Continuous Testing

```bash
# Watch mode for development
cargo watch -x 'run --bin service'

# Run tests
cargo test

# Check code
cargo clippy
cargo fmt --check
```

## API Testing with Postman/Insomnia

Import this collection:

```json
{
  "name": "RustIngester API",
  "requests": [
    {
      "name": "Health Check",
      "method": "GET",
      "url": "http://localhost:3000/status"
    },
    {
      "name": "Ingest Batch",
      "method": "POST",
      "url": "http://localhost:3000/ingest/batch",
      "body": "@Data/ok.json"
    },
    {
      "name": "Query Similar",
      "method": "POST",
      "url": "http://localhost:3000/query/similar",
      "body": {
        "query": "user python package",
        "top_k": 5
      }
    }
  ]
}
```

---

**Happy Testing! 🚀**
