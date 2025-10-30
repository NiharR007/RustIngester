# RustIngester Service Implementation Guide

## Overview

This guide documents the implementation of a REST API service for ingesting knowledge graphs from the `ok.json` format into PostgreSQL with Apache AGE.

## ok.json Data Structure

### Format
```json
{
  "session_id_uuid": {
    "nodes": [
      {"id": "node_identifier", "type": "NodeLabel"}
    ],
    "edges": [
      {
        "source": "source_node_id",
        "relation": "relationship_type",
        "target": "target_node_id",
        "evidence_message_ids": ["uuid1", "uuid2"]
      }
    ]
  }
}
```

### Key Characteristics
- **Session-based**: Each UUID key represents a conversation/session
- **Node structure**: `id` (unique identifier) + `type` (AGE label)
- **Edge structure**: source/target references + relation + evidence tracking
- **Evidence tracking**: Array of message IDs for provenance

## Architecture Changes

### 1. Data Models (src/etl/parser.rs)

**New Models:**
```rust
// Node from ok.json
pub struct KnowledgeNode {
    pub id: String,
    pub node_type: String,  // Maps to AGE label
}

// Edge from ok.json
pub struct KnowledgeEdge {
    pub source: String,
    pub relation: String,
    pub target: String,
    pub evidence_message_ids: Vec<String>,
}

// Session graph
pub struct SessionGraph {
    pub nodes: Vec<KnowledgeNode>,
    pub edges: Vec<KnowledgeEdge>,
}

// Complete file structure
pub type KnowledgeGraphData = HashMap<String, SessionGraph>;
```

**Conversion to existing models:**
- `KnowledgeNode` → `ParsedNode` (label = type, pk = id)
- `KnowledgeEdge` → edge properties with evidence tracking

### 2. Batch Ingestion (src/ingest.rs)

**New Functions:**
```rust
// Ingest a single session graph
pub async fn ingest_session_graph(
    session_id: &str,
    graph: &SessionGraph
) -> Result<SessionIngestStats>

// Ingest entire ok.json file
pub async fn ingest_knowledge_graph_file(
    file_path: &str
) -> Result<BatchIngestStats>
```

**Process:**
1. Parse ok.json file
2. For each session:
   - Create all nodes (upsert by id)
   - Create all edges with evidence properties
   - Generate embeddings for each edge
   - Store in LSH buckets
3. Return statistics

### 3. Web Service (src/api/)

**Dependencies (Cargo.toml):**
```toml
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.0", features = ["serde"] }
```

**Module Structure:**
```
src/api/
├── mod.rs           # Module exports
├── handlers.rs      # Request handlers
├── models.rs        # Request/Response DTOs
└── routes.rs        # Route definitions
```

### 4. API Endpoints

#### GET /status
**Purpose:** Health check and system status

**Response:**
```json
{
  "status": "healthy",
  "database": "connected",
  "age_extension": "loaded",
  "graph_name": "sem_graph",
  "total_nodes": 1234,
  "total_edges": 5678,
  "total_sessions": 10
}
```

#### POST /ingest/session
**Purpose:** Ingest a single session graph

**Request:**
```json
{
  "session_id": "688e7460-8e78-800d-bccb-7d9d5380dc33",
  "graph": {
    "nodes": [...],
    "edges": [...]
  }
}
```

**Response:**
```json
{
  "session_id": "688e7460-8e78-800d-bccb-7d9d5380dc33",
  "nodes_created": 4,
  "edges_created": 4,
  "embeddings_created": 4,
  "duration_ms": 123
}
```

#### POST /ingest/batch
**Purpose:** Ingest entire ok.json file

**Request:**
```json
{
  "sessions": {
    "session_id_1": {"nodes": [...], "edges": [...]},
    "session_id_2": {"nodes": [...], "edges": [...]}
  }
}
```

**Response:**
```json
{
  "total_sessions": 10,
  "total_nodes": 150,
  "total_edges": 300,
  "total_embeddings": 300,
  "duration_ms": 5432,
  "errors": []
}
```

#### POST /query/similar
**Purpose:** Query similar triplets/edges

**Request:**
```json
{
  "query": "user wants to install python package",
  "top_k": 5,
  "threshold": 0.7
}
```

**Response:**
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
  ]
}
```

#### GET /query/session/{session_id}
**Purpose:** Retrieve complete session graph

**Response:**
```json
{
  "session_id": "688e7460-8e78-800d-bccb-7d9d5380dc33",
  "nodes": [...],
  "edges": [...]
}
```

#### POST /graph/cypher
**Purpose:** Execute custom Cypher queries

**Request:**
```json
{
  "query": "MATCH (n:Person)-[r]->(m) RETURN n, r, m LIMIT 10"
}
```

**Response:**
```json
{
  "results": [...],
  "count": 10
}
```

## Database Schema Changes

### Evidence Tracking Table
```sql
CREATE TABLE IF NOT EXISTS edge_evidence (
    edge_id BIGINT,
    session_id TEXT,
    evidence_message_id TEXT,
    PRIMARY KEY (edge_id, evidence_message_id)
);
```

### Session Metadata Table
```sql
CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    ingested_at TIMESTAMP DEFAULT NOW(),
    node_count INTEGER,
    edge_count INTEGER
);
```

### Updated Embeddings Table
```sql
ALTER TABLE embeddings ADD COLUMN IF NOT EXISTS session_id TEXT;
ALTER TABLE embeddings ADD COLUMN IF NOT EXISTS edge_text TEXT;
```

## Implementation Steps

### Phase 1: Data Models & Parsing ✓
1. ✅ Add new structs to `src/etl/parser.rs`
2. ✅ Implement JSON deserialization
3. ✅ Add conversion functions to existing models

### Phase 2: Database Layer ✓
1. ✅ Update `src/db/connect.rs` to create new tables
2. ✅ Add evidence tracking functions to `src/db/graph.rs`
3. ✅ Update vector storage for session tracking

### Phase 3: Batch Ingestion ✓
1. ✅ Implement `ingest_session_graph()` in `src/ingest.rs`
2. ✅ Implement `ingest_knowledge_graph_file()`
3. ✅ Add progress tracking and error handling

### Phase 4: Web Service ✓
1. ✅ Add dependencies to `Cargo.toml`
2. ✅ Create `src/api/` module
3. ✅ Implement handlers for all endpoints
4. ✅ Add CORS and logging middleware

### Phase 5: Testing ✓
1. ✅ Test with ok.json file
2. ✅ Verify graph structure in database
3. ✅ Test similarity queries
4. ✅ Load testing with extensive data

## Usage Examples

### Starting the Service
```bash
# Set environment variables
export DATABASE_URL="postgresql://postgres:password@localhost:5432/postgres"
export LSH_BUCKETS=128
export SERVER_PORT=3000

# Run the service
cargo run --release --bin service
```

### Ingesting ok.json
```bash
# Using curl
curl -X POST http://localhost:3000/ingest/batch \
  -H "Content-Type: application/json" \
  -d @Data/ok.json

# Using the CLI (alternative)
cargo run --release --bin ingest -- --file Data/ok.json
```

### Querying Similar Edges
```bash
curl -X POST http://localhost:3000/query/similar \
  -H "Content-Type: application/json" \
  -d '{
    "query": "user requests help with python",
    "top_k": 5
  }'
```

### Health Check
```bash
curl http://localhost:3000/status
```

## Performance Considerations

### Batch Ingestion Optimization
- **Parallel processing**: Process sessions concurrently
- **Batch commits**: Group database operations
- **Connection pooling**: Use `deadpool-postgres`

### Embedding Generation
- **Current**: Placeholder (0.1 vector)
- **TODO**: Integrate real embedding model
- **Options**:
  - OpenAI API
  - Local Sentence Transformers
  - HuggingFace Inference API

### LSH Configuration
- **Small datasets** (< 1000 edges): 128 buckets
- **Medium datasets** (1000-10000): 256 buckets
- **Large datasets** (> 10000): 512 buckets

## Testing Strategy

### Unit Tests
- Parser: ok.json → data models
- Conversion: KnowledgeNode → ParsedNode
- Evidence tracking storage/retrieval

### Integration Tests
- Full session ingestion
- Query with evidence retrieval
- Batch processing

### Load Tests
- Ingest 1000+ sessions
- Concurrent query performance
- Memory usage monitoring

## Deployment

### Docker Setup
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libpq5
COPY --from=builder /app/target/release/service /usr/local/bin/
CMD ["service"]
```

### Environment Variables
```bash
DATABASE_URL=postgresql://user:pass@host:5432/db
LSH_BUCKETS=128
SERVER_PORT=3000
LOG_LEVEL=info
```

## Monitoring

### Metrics to Track
- Ingestion rate (sessions/sec)
- Query latency (p50, p95, p99)
- Database connection pool usage
- Memory consumption
- Error rates

### Logging
- Request/response logging
- Database query logging
- Error stack traces
- Performance metrics

## Future Enhancements

1. **Streaming Ingestion**: WebSocket endpoint for real-time updates
2. **Graph Analytics**: PageRank, community detection
3. **Advanced Queries**: Graph traversal API
4. **Caching**: Redis for frequently accessed sessions
5. **Authentication**: JWT-based API authentication
6. **Rate Limiting**: Per-client request limits
7. **Webhooks**: Notify on ingestion completion
8. **Export**: Export sessions to various formats

## Troubleshooting

### Common Issues

**Issue**: "extension 'age' does not exist"
- **Solution**: Install Apache AGE and restart PostgreSQL

**Issue**: "connection refused"
- **Solution**: Check DATABASE_URL and PostgreSQL status

**Issue**: "label already exists"
- **Solution**: Service handles this automatically via upsert

**Issue**: Slow ingestion
- **Solution**: Increase LSH_BUCKETS or use connection pooling

## References

- [Apache AGE Documentation](https://age.apache.org/)
- [Axum Web Framework](https://docs.rs/axum/)
- [tokio-postgres](https://docs.rs/tokio-postgres/)
- [LSH Algorithm](https://en.wikipedia.org/wiki/Locality-sensitive_hashing)

---

**Last Updated**: 2025-10-29
**Version**: 1.0.0
