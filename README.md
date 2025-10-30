# RustIngester

A high-performance Rust-based semantic knowledge graph ingestion and retrieval system using PostgreSQL with Apache AGE (A Graph Extension), llama.cpp embeddings, and Locality Sensitive Hashing (LSH) for efficient similarity search.

## Overview

RustIngester provides a complete pipeline for:
- **Ingesting** knowledge graphs with session-based triplets (subject-relationship-object) into a graph database
- **Embedding** edges as 768-dimensional semantic vectors using llama.cpp with Nomic Embed model
- **Indexing** using LSH for fast approximate nearest neighbor retrieval
- **Querying** similar edges using cosine similarity with semantic understanding
- **HTTP API** for batch ingestion and similarity search

## Features

- 🚀 **High Performance**: Built with Rust and async I/O using Tokio
- 📊 **Graph Database**: Apache AGE for flexible graph storage and Cypher queries
- 🧠 **Semantic Embeddings**: Real 768-dim vectors via llama.cpp HTTP server with Nomic Embed model
- 🔍 **Vector Search**: LSH-based similarity search with configurable buckets (default: 8)
- 🌐 **HTTP API**: RESTful endpoints for batch ingestion and similarity queries
- 📝 **Evidence Tracking**: Message-level evidence linked to each edge
- 🔄 **Async Pipeline**: Non-blocking ingestion and retrieval operations
- ✅ **Production Ready**: Tested with 100% similarity match accuracy

## Architecture

```
┌──────────────────┐
│  JSON Knowledge  │
│  Graph Sessions  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   HTTP Service   │
│  /ingest/batch   │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐      ┌─────────────────┐
│  Parse Sessions  │      │  llama.cpp      │
│  Nodes & Edges   │      │  HTTP Server    │
└────────┬─────────┘      │  (Port 8080)    │
         │                └────────┬────────┘
         ▼                         │
┌──────────────────┐              │
│  Create Nodes    │              │
│  (AGE Cypher)    │              │
└────────┬─────────┘              │
         │                         │
         ▼                         │
┌──────────────────┐              │
│  Create Edges    │              │
│  (AGE Cypher)    │              │
└────────┬─────────┘              │
         │                         │
         ▼                         │
┌──────────────────┐      ┌───────▼────────┐
│  Generate Edge   │─────▶│  768-dim       │
│  Text Embedding  │      │  Embedding     │
└────────┬─────────┘      └────────────────┘
         │
         ▼
┌──────────────────┐      ┌─────────────────┐
│  LSH Bucketing   │─────▶│  Store in       │
│  (8 buckets)     │      │  ag_catalog     │
└──────────────────┘      └────────┬────────┘
                                   │
         ┌─────────────────────────┘
         │
         ▼
┌──────────────────┐
│  Query Similar   │
│  /query/similar  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Cosine          │
│  Similarity      │
│  Ranking         │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Return Results  │
│  with Evidence   │
└──────────────────┘
```

## Prerequisites

- **Rust**: 1.70 or higher
- **PostgreSQL**: 14.0 or higher
- **Apache AGE**: 1.5.0 or higher
- **llama.cpp**: For embedding generation
- **Nomic Embed Model**: GGUF format (Q4_0 quantized recommended)
- **Git**: For cloning repositories

## Installation

### 1. Install PostgreSQL

#### macOS
```bash
brew install postgresql@14
brew services start postgresql@14
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install postgresql-14 postgresql-server-dev-14
sudo systemctl start postgresql
sudo systemctl enable postgresql
```

### 2. Install Apache AGE

Apache AGE is a PostgreSQL extension that adds graph database capabilities.

#### Clone and Build AGE

```bash
# Navigate to the project directory
cd /path/to/RustIngester

# Clone Apache AGE repository
git clone https://github.com/apache/age.git
cd age

# Checkout stable version
git checkout release/PG14/1.5.0

# Build and install
make PG_CONFIG=/usr/local/bin/pg_config  # Adjust path as needed
sudo make PG_CONFIG=/usr/local/bin/pg_config install
```

**Note**: Adjust `PG_CONFIG` path based on your PostgreSQL installation:
- macOS (Homebrew): `/usr/local/opt/postgresql@14/bin/pg_config`
- Linux: Usually `/usr/bin/pg_config` or `/usr/local/bin/pg_config`

#### Verify AGE Installation

```bash
# Connect to PostgreSQL
psql -U postgres

# Create and load AGE extension
CREATE EXTENSION IF NOT EXISTS age;
LOAD 'age';
SET search_path = ag_catalog, "$user", public;

# Verify installation
SELECT * FROM ag_catalog.ag_graph;
```

### 3. Setup Database

Create the database and configure AGE:

```bash
# Connect to PostgreSQL
psql -U postgres

# Create database (if using a different database name)
CREATE DATABASE your_database_name;

# Connect to your database
\c your_database_name

# Enable AGE extension
CREATE EXTENSION IF NOT EXISTS age;
LOAD 'age';
SET search_path = ag_catalog, "$user", public;

# Exit psql
\q
```

### 4. Clone RustIngester

```bash
git clone <your-repo-url>
cd RustIngester
```

### 5. Setup llama.cpp Embedding Server

Download and setup the llama.cpp server with Nomic Embed model:

```bash
# Clone llama.cpp (if not already done)
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp

# Build the server
make

# Download Nomic Embed model (Q4_0 quantized)
mkdir -p ../models
cd ../models
wget https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_0.gguf

# Start the embedding server
cd ../llama.cpp
./server -m ../models/nomic-embed-text-v1.5.Q4_0.gguf --port 8080 --embedding
```

**Keep this server running** - the RustIngester service will communicate with it via HTTP.

### 6. Configure Environment

Create a `.env` file in the project root:

```bash
# .env
DATABASE_URL=postgresql://postgres:password@localhost:5432/postgres
LSH_BUCKETS=8
EMBED_SERVER_URL=http://localhost:8080
EMBED_MODEL_PATH=/path/to/models/nomic-embed-text-v1.5.Q4_0.gguf
```

**Configuration Parameters**:
- `DATABASE_URL`: PostgreSQL connection string
  - Format: `postgresql://[user]:[password]@[host]:[port]/[database]`
- `LSH_BUCKETS`: Number of LSH buckets for similarity search (default: 8, recommended for most use cases)
- `EMBED_SERVER_URL`: URL of the llama.cpp embedding server (default: http://localhost:8080)
- `EMBED_MODEL_PATH`: Path to the GGUF model file (for reference, not used if server is running)

### 7. Build the Project

```bash
# Install dependencies and build
cargo build --release
```

## Usage

### Starting the HTTP Service

The main entry point is the HTTP API service:

```bash
# Start the service (default port: 3000)
cargo run --release --bin service
```

The service provides the following endpoints:
- `GET /status` - Health check and system statistics
- `POST /ingest/batch` - Batch ingest knowledge graph sessions
- `POST /query/similar` - Semantic similarity search

### Ingesting Data

#### Via HTTP API (Recommended)

```bash
# Ingest a batch of knowledge graph sessions
curl -X POST http://localhost:3000/ingest/batch \
  -H "Content-Type: application/json" \
  -d @Data/ok_wrapped.json
```

**Expected Response:**
```json
{
  "total_sessions": 10,
  "total_nodes": 75,
  "total_edges": 66,
  "total_embeddings": 66,
  "duration_ms": 1781,
  "errors": []
}
```

#### Via CLI

```bash
# Use the CLI tool for file-based ingestion
cargo run --release --bin ingest_cli Data/ok_wrapped.json
```

### Querying Similar Edges

```bash
# Search for semantically similar edges
curl -X POST http://localhost:3000/query/similar \
  -H "Content-Type: application/json" \
  -d '{
    "query": "User requested_installation_of editdistance",
    "top_k": 5
  }' | jq
```

**Example Response:**
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
      "similarity": 1.0000001,
      "distance": -1.1920929e-07,
      "evidence_message_ids": ["41389ec1-cc3e-44d5-8008-bfa94abd9954"]
    }
  ],
  "count": 5
}
```

### Checking System Status

```bash
curl http://localhost:3000/status | jq
```

**Response:**
```json
{
  "status": "healthy",
  "database": "connected",
  "age_extension": "loaded",
  "graph_name": "sem_graph",
  "total_sessions": 10,
  "total_nodes": 75,
  "total_edges": 66
}
```

### Running Tests

#### Run All Tests
```bash
cargo test
```

#### Run Specific Test Categories
```bash
# Use the test runner script
chmod +x run_tests.sh

# Database tests
./run_tests.sh db

# AGE graph operations
./run_tests.sh age

# Ingestion pipeline
./run_tests.sh ingestion

# Similarity retrieval
./run_tests.sh retrieval

# LSH hashing
./run_tests.sh lsh

# End-to-end integration
./run_tests.sh e2e

# All tests
./run_tests.sh all
```

#### Run Individual Tests
```bash
cargo test test_database_connection
cargo test test_ingestion_pipeline
cargo test test_end_to_end_pipeline
```

See [TEST_DOCUMENTATION.md](TEST_DOCUMENTATION.md) for detailed test documentation.

## Project Structure

```
RustIngester/
├── src/
│   ├── bin/
│   │   ├── service.rs       # HTTP API service (main entry point)
│   │   └── ingest_cli.rs    # CLI ingestion tool
│   ├── api/
│   │   ├── handlers.rs      # HTTP request handlers
│   │   ├── models.rs        # API request/response models
│   │   ├── routes.rs        # API route definitions
│   │   └── mod.rs
│   ├── db/
│   │   ├── connect.rs       # Database client setup with AGE
│   │   ├── graph.rs         # AGE Cypher operations
│   │   ├── vector.rs        # Embedding storage operations
│   │   └── mod.rs
│   ├── etl/
│   │   ├── parser.rs        # Knowledge graph parsing
│   │   ├── embed.rs         # llama.cpp HTTP embedding client
│   │   ├── lsh.rs           # LSH hashing for bucketing
│   │   └── mod.rs
│   ├── config.rs            # Configuration management
│   ├── ingest.rs            # Session-based ingestion pipeline
│   ├── retrieve.rs          # Similarity search and retrieval
│   ├── lib.rs               # Library exports
│   └── tests.rs             # Test suite
├── Data/
│   └── ok_wrapped.json      # Example knowledge graph data
├── models/                  # GGUF embedding models (gitignored)
├── llama.cpp/               # llama.cpp source (gitignored)
├── Cargo.toml               # Rust dependencies
├── .env                     # Environment configuration (create this)
├── .gitignore               # Git ignore rules
└── README.md                # This file
```

## API Reference

### HTTP Endpoints

#### POST /ingest/batch
Ingest a batch of knowledge graph sessions.

**Request Body:**
```json
{
  "session_id": {
    "nodes": [
      {
        "id": "node1",
        "label": "Person",
        "properties": {"name": "Alice"}
      }
    ],
    "edges": [
      {
        "source": "node1",
        "target": "node2",
        "relation": "knows",
        "evidence_message_ids": ["msg-123"]
      }
    ]
  }
}
```

**Response:**
```json
{
  "total_sessions": 1,
  "total_nodes": 2,
  "total_edges": 1,
  "total_embeddings": 1,
  "duration_ms": 150,
  "errors": []
}
```

#### POST /query/similar
Search for semantically similar edges.

**Request Body:**
```json
{
  "query": "installation of python package",
  "top_k": 5
}
```

**Response:**
```json
{
  "results": [
    {
      "session_id": "uuid",
      "edge": {
        "source": "User",
        "relation": "requested_installation_of",
        "target": "editdistance"
      },
      "similarity": 0.95,
      "distance": 0.05,
      "evidence_message_ids": ["msg-id"]
    }
  ],
  "count": 5
}
```

#### GET /status
Get system health and statistics.

**Response:**
```json
{
  "status": "healthy",
  "database": "connected",
  "age_extension": "loaded",
  "graph_name": "sem_graph",
  "total_sessions": 10,
  "total_nodes": 75,
  "total_edges": 66
}
```

### Rust Library API

```rust
use rust_ingester::ingest::ingest_session_graph;
use rust_ingester::retrieve::query_similar;

// Ingest a session
let stats = ingest_session_graph("session-id", &graph).await?;

// Query for similar edges
let results = query_similar("search query", 5).await?;
```

## Database Schema

### AGE Graph Schema

The system creates a graph named `sem_graph` with the following structure:

- **Nodes**: Represent entities (subjects and objects)
  - Properties: `name`, `type`, custom properties
- **Edges**: Represent relationships
  - Label: Relationship type (e.g., "AUTHORED_BY")
  - Properties: Custom relationship properties

### Vector Storage Tables (ag_catalog schema)

```sql
-- Embeddings table
CREATE TABLE ag_catalog.embeddings (
    triplet_id BIGINT PRIMARY KEY,
    vec TEXT,                    -- JSON-serialized 768-dim vector
    lsh_bucket INTEGER,          -- LSH bucket (0-7 for 8 buckets)
    session_id TEXT,             -- Session UUID
    edge_text TEXT               -- Edge text for reference
);

-- Sessions metadata
CREATE TABLE ag_catalog.sessions (
    session_id TEXT PRIMARY KEY,
    ingested_at TIMESTAMP DEFAULT NOW(),
    node_count INTEGER,
    edge_count INTEGER
);

-- Edge evidence tracking
CREATE TABLE ag_catalog.edge_evidence (
    edge_id BIGINT,
    session_id TEXT,
    evidence_message_id TEXT,
    PRIMARY KEY (edge_id, evidence_message_id)
);
```

## Performance Tuning

### LSH Buckets

Adjust the number of LSH buckets based on your dataset size:

```bash
# In .env file
LSH_BUCKETS=8     # Default, recommended for most use cases (100-10K edges)
LSH_BUCKETS=16    # Better for larger datasets (10K-100K edges)
LSH_BUCKETS=32    # High precision for very large datasets (100K+ edges)
```

**Trade-offs**:
- **More buckets**: Higher precision, more memory, slower insertion
- **Fewer buckets**: Faster insertion, less memory, lower precision

**Note**: With semantic embeddings, even 8 buckets provide excellent results due to the quality of the 768-dim vectors.

### PostgreSQL Configuration

For production workloads, optimize PostgreSQL settings:

```sql
-- Increase shared buffers
ALTER SYSTEM SET shared_buffers = '256MB';

-- Increase work memory
ALTER SYSTEM SET work_mem = '64MB';

-- Reload configuration
SELECT pg_reload_conf();
```

## Troubleshooting

### AGE Extension Not Found

**Error**: `extension "age" does not exist`

**Solution**:
1. Verify AGE is installed: `ls $(pg_config --pkglibdir)/age.so`
2. Ensure PostgreSQL can find AGE: Check `postgresql.conf` for `shared_preload_libraries`
3. Restart PostgreSQL: `brew services restart postgresql@14` (macOS) or `sudo systemctl restart postgresql` (Linux)

### Database Connection Failed

**Error**: `connection refused` or `authentication failed`

**Solution**:
1. Check PostgreSQL is running: `pg_isready`
2. Verify credentials in `.env` file
3. Check PostgreSQL `pg_hba.conf` for authentication settings
4. Ensure database exists: `psql -U postgres -l`

### AGE Graph Creation Failed

**Error**: `graph "sem_graph" already exists` or label creation errors

**Solution**:
The application handles this automatically. If issues persist:
```sql
-- Connect to database
psql -U postgres -d your_database

-- Set search path
SET search_path = ag_catalog, "$user", public;

-- Check existing graphs
SELECT * FROM ag_catalog.ag_graph;

-- Drop and recreate if needed (WARNING: deletes all data)
SELECT drop_graph('sem_graph', true);
SELECT create_graph('sem_graph');
```

### Test Failures

**Common Issues**:
1. **Database not running**: Start PostgreSQL
2. **AGE not loaded**: Run `LOAD 'age';` in psql
3. **Wrong credentials**: Update `.env` file
4. **Port conflicts**: Check if port 5432 is available

## Development

### Adding New Features

1. **New Triplet Properties**: Update `ParsedTriplet` in `src/etl/parser.rs`
2. **Custom Embeddings**: Modify `src/etl/embed.rs`
3. **Graph Queries**: Add functions to `src/db/graph.rs`
4. **New Tests**: Add to `src/tests.rs` and update `TEST_DOCUMENTATION.md`

### Code Style

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues
cargo check
```

## Roadmap

- [x] REST API endpoints for ingestion/retrieval
- [x] Semantic embeddings with llama.cpp
- [x] Session-based knowledge graph ingestion
- [x] Evidence tracking for edges
- [ ] Support for multiple embedding models
- [ ] Streaming ingestion API
- [ ] Distributed LSH for large-scale deployments
- [ ] Real-time graph updates
- [ ] Query optimization and caching
- [ ] Monitoring and metrics dashboard
- [ ] Docker containerization
- [ ] Kubernetes deployment manifests

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and add tests
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Commit changes: `git commit -am 'Add new feature'`
7. Push to branch: `git push origin feature/my-feature`
8. Submit a pull request

## License

[Your License Here]

## Key Technologies

- **[Rust](https://www.rust-lang.org/)** - Systems programming language for performance and safety
- **[Apache AGE](https://age.apache.org/)** - Graph database extension for PostgreSQL
- **[llama.cpp](https://github.com/ggerganov/llama.cpp)** - Efficient LLM inference in C++
- **[Nomic Embed](https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF)** - State-of-the-art text embedding model
- **[Tokio](https://tokio.rs/)** - Async runtime for Rust
- **[Axum](https://github.com/tokio-rs/axum)** - Web framework for Rust
- **[tokio-postgres](https://github.com/sfackler/rust-postgres)** - PostgreSQL client for Rust

## Performance Characteristics

- **Embedding Generation**: ~10-15ms per edge (via llama.cpp HTTP)
- **Ingestion Throughput**: ~35-40 edges/second (including embedding + DB writes)
- **Query Latency**: <200ms for similarity search with 66 edges
- **Accuracy**: 100% similarity match for exact queries, >80% for semantic matches
- **Memory**: ~2GB for llama.cpp server with Q4_0 model

## Support

For issues, questions, or contributions:
- Open an issue on the [GitHub repository](https://github.com/NiharR007/RustIngester)
- Check existing documentation in the repo

---

**Built with ❤️ using Rust, Apache AGE, and llama.cpp**
