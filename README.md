# RustIngester

A high-performance Rust-based **RAG (Retrieval-Augmented Generation)** system for conversational AI, combining semantic message search with knowledge graph retrieval using PostgreSQL, pgvector, Apache AGE, and llama.cpp embeddings.

## Overview

RustIngester provides a complete RAG pipeline for conversational context retrieval:
- **Ingesting** conversation messages with 768-dimensional semantic embeddings
- **Storing** knowledge graphs with conversation-aware nodes and edges
- **Semantic Search** using pgvector for cosine similarity retrieval
- **Context Retrieval** for LLM prompts with relevance scoring
- **HTTP API** for ingestion, querying, and LLM context generation

## Features

- 🚀 **High Performance**: Built with Rust and async I/O using Tokio
- 💬 **Message-Level RAG**: Store and retrieve full conversation messages with embeddings
- 📊 **Dual Storage**: pgvector for semantic search + Apache AGE for knowledge graphs
- 🧠 **Semantic Embeddings**: 768-dim vectors via llama.cpp with Nomic Embed v1.5
- 🔍 **Vector Search**: Native pgvector cosine similarity with IVFFlat indexing
- 🌐 **Production API**: RESTful endpoints for ingestion and LLM context retrieval
- 📝 **Evidence Tracking**: Link knowledge graph edges to source messages
- 🔄 **Async Pipeline**: Non-blocking ingestion and retrieval operations
- ✅ **Battle Tested**: Successfully ingested 5,741 messages + 270 conversations

## Quick Start

### 🐳 Option 1: Docker (Recommended - 2 minutes)

```bash
# 1. Clone repository
git clone <your-repo-url>
cd RustIngester

# 2. Download model (one-time, ~74MB)
./download-model.sh

# 3. Start everything
docker compose up -d

# 4. Test the API
curl http://localhost:3000/status
```

**That's it!** See [QUICKSTART_DOCKER.md](QUICKSTART_DOCKER.md) for details.

### 🛠️ Option 2: Manual Setup (30+ minutes)

<details>
<summary>Click to expand manual installation steps</summary>

```bash
# 1. Install dependencies (PostgreSQL 14, pgvector, Apache AGE)
brew install postgresql@14
brew services start postgresql@14

# 2. Clone and setup
git clone <your-repo-url>
cd RustIngester

# 3. Setup database
psql postgres -c "CREATE EXTENSION IF NOT EXISTS vector;"
psql postgres -c "CREATE EXTENSION IF NOT EXISTS age;"

# 4. Configure environment
cat > .env << EOF
DATABASE_URL=postgresql://$(whoami)@localhost:5432/postgres
LSH_BUCKETS=8
SERVER_PORT=3000
EMBED_SERVER_URL=http://localhost:8080
EMBED_MODEL_PATH=/path/to/models/nomic-embed-text-v1.5.Q4_0.gguf
EOF

# 5. Start llama.cpp embedding server (in background)
cd llama.cpp
./build/bin/llama-server -m ../models/nomic-embed-text-v1.5.Q4_0.gguf --port 8080 --embeddings &
cd ..

# 6. Build and run
cargo build --release
cargo run --release --bin service

# 7. Test the API
curl http://localhost:3000/status
```

See full manual installation guide below.
</details>

## Architecture

```
┌──────────────────────┐     ┌──────────────────────┐
│  Turn Embeddings     │     │  Knowledge Graph     │
│  (Messages + Embeds) │     │  (Nodes + Edges)     │
└──────────┬───────────┘     └──────────┬───────────┘
           │                            │
           ▼                            ▼
    ┌───────────────────────────────────────────┐
    │        HTTP Service (Port 3000)           │
    │  /ingest/messages    /ingest/knowledge-graph  │
    └─────────────┬─────────────────────────────┘
                  │
                  ▼
    ┌─────────────────────────────────────────┐
    │         PostgreSQL Database             │
    │  ┌─────────────┐    ┌─────────────┐    │
    │  │  pgvector   │    │ Apache AGE  │    │
    │  │  Messages   │    │ Knowledge   │    │
    │  │  Embeddings │    │ Graph       │    │
    │  └─────────────┘    └─────────────┘    │
    └─────────────────────────────────────────┘
                  │
                  ▼
    ┌─────────────────────────────────────────┐
    │         Query / Retrieval               │
    │                                         │
    │  /query/llm-context                    │
    │  ┌──────────────────────────────────┐  │
    │  │ 1. Generate Query Embedding      │  │
    │  │    (llama.cpp Port 8080)         │  │
    │  └──────────────┬───────────────────┘  │
    │                 │                       │
    │  ┌──────────────▼───────────────────┐  │
    │  │ 2. Semantic Search (pgvector)    │  │
    │  │    Cosine Similarity on          │  │
    │  │    Message Embeddings            │  │
    │  └──────────────┬───────────────────┘  │
    │                 │                       │
    │  ┌──────────────▼───────────────────┐  │
    │  │ 3. Optional KG Context           │  │
    │  │    (Apache AGE Cypher)           │  │
    │  └──────────────┬───────────────────┘  │
    │                 │                       │
    │  ┌──────────────▼───────────────────┐  │
    │  │ 4. Format for LLM                │  │
    │  │    - Relevance Scores            │  │
    │  │    - Token Budget Management     │  │
    │  │    - Conversation Context        │  │
    │  └──────────────────────────────────┘  │
    └─────────────────────────────────────────┘
```

## Prerequisites

- **Rust**: 1.70 or higher
- **PostgreSQL**: 14.0 or higher
- **pgvector**: 0.8.0 or higher (for semantic search)
- **Apache AGE**: 1.5.0 or higher (for knowledge graphs)
- **llama.cpp**: For embedding generation (query time)
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

### 2. Install pgvector Extension

pgvector is required for efficient semantic similarity search on embeddings.

#### macOS
```bash
# Clone pgvector
git clone --branch v0.8.0 https://github.com/pgvector/pgvector.git
cd pgvector

# Build and install
make PG_CONFIG=/opt/homebrew/opt/postgresql@14/bin/pg_config
make install PG_CONFIG=/opt/homebrew/opt/postgresql@14/bin/pg_config
```

#### Linux
```bash
# Clone pgvector
git clone --branch v0.8.0 https://github.com/pgvector/pgvector.git
cd pgvector

# Build and install
make
sudo make install
```

#### Verify pgvector Installation
```bash
psql postgres -c "CREATE EXTENSION IF NOT EXISTS vector;"
psql postgres -c "SELECT * FROM pg_extension WHERE extname = 'vector';"
```

### 3. Install Apache AGE

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

### 4. Setup Database

Create the database and configure extensions:

```bash
# Connect to PostgreSQL
psql -U postgres

# Create database (if using a different database name)
CREATE DATABASE your_database_name;

# Connect to your database
\c your_database_name

# Enable extensions
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS age;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
LOAD 'age';
SET search_path = ag_catalog, "$user", public;

# Exit psql
\q
```

### 5. Clone RustIngester

```bash
git clone <your-repo-url>
cd RustIngester
```

### 6. Setup llama.cpp Embedding Server

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

# Start the embedding server (run in background)
cd ../llama.cpp
./build/bin/llama-server -m ../models/nomic-embed-text-v1.5.Q4_0.gguf --port 8080 --embeddings -ngl 1 -c 2048 &
```

**Keep this server running** - it generates embeddings for queries at runtime.

### 7. Configure Environment

Create a `.env` file in the project root:

```bash
# .env
DATABASE_URL=postgresql://your_username@localhost:5432/postgres
LSH_BUCKETS=8
SERVER_PORT=3000
EMBED_MODEL_PATH=/path/to/RustIngester/models/nomic-embed-text-v1.5.Q4_0.gguf
EMBED_SERVER_URL=http://localhost:8080
```

**Configuration Parameters**:
- `DATABASE_URL`: PostgreSQL connection string
  - Format: `postgresql://[user]:[password]@[host]:[port]/[database]`
  - For local Homebrew PostgreSQL without password: `postgresql://your_username@localhost:5432/postgres`
- `LSH_BUCKETS`: Number of LSH buckets (default: 8, for legacy edge similarity)
- `SERVER_PORT`: HTTP API port (default: 3000)
- `EMBED_SERVER_URL`: URL of the llama.cpp embedding server
- `EMBED_MODEL_PATH`: Path to the GGUF model file

### 8. Build the Project

```bash
# Install dependencies and build
cargo build --release
```

## Usage

### Starting the Services

#### 1. Start llama.cpp Embedding Server (if not already running)
```bash
cd llama.cpp
./build/bin/llama-server -m ../models/nomic-embed-text-v1.5.Q4_0.gguf --port 8080 --embeddings -ngl 1 -c 2048 > /tmp/llama-embed-server.log 2>&1 &
```

#### 2. Start RustIngester Service
```bash
# Start the service (default port: 3000)
cargo run --release --bin service
```

The service provides the following endpoints:
- `GET  /status` - Health check and system statistics
- `POST /ingest/messages` - Ingest conversation messages with embeddings
- `POST /ingest/knowledge-graph` - Ingest knowledge graph nodes and edges
- `GET  /ingest/statistics` - Get ingestion statistics
- `POST /query/llm-context` - Query for LLM context (RAG retrieval)
- `POST /query/messages` - Get messages by IDs
- `POST /query/similar` - Legacy edge similarity search
- `POST /graph/cypher` - Execute custom Cypher queries

### Ingesting Data

#### 1. Ingest Conversation Messages with Embeddings

```bash
# Ingest messages with pre-computed embeddings
curl -X POST http://localhost:3000/ingest/messages \
  -H "Content-Type: application/json" \
  -d @Data/turn_embeddings.json

# Expected response
{
  "success": true,
  "total_processed": 5741,
  "total_inserted": 5741,
  "duration_ms": 3547,
  "errors": []
}
```

**Input Format** (`turn_embeddings.json`):
```json
[
  {
    "message_id": "41389ec1-cc3e-44d5-8008-bfa94abd9954",
    "conversation_id": "688e7460-8e78-800d-bccb-7d9d5380dc33",
    "actual_text": "user: pip install editdistance",
    "embedding": [0.012, 0.002, ..., -0.056]  // 768-dim vector
  }
]
```

#### 2. Ingest Knowledge Graph

```bash
# Ingest knowledge graph nodes and edges
curl -X POST http://localhost:3000/ingest/knowledge-graph \
  -H "Content-Type: application/json" \
  -d @Data/enhanced_pipeline_full_results.json

# Expected response
{
  "success": true,
  "total_processed": 3329,
  "total_inserted": 3329,
  "duration_ms": 963,
  "errors": []
}
```

**Input Format** (`enhanced_pipeline_full_results.json`):
```json
{
  "conversation-uuid": {
    "nodes": [
      {"id": "user", "type": "Person"},
      {"id": "install_package", "type": "Action"}
    ],
    "edges": [
      {
        "source": "user",
        "target": "install_package",
        "relation": "wants_to",
        "evidence_message_ids": ["41389ec1-cc3e-44d5-8008-bfa94abd9954"]
      }
    ]
  }
}
```

### Querying for LLM Context (RAG Retrieval)

```bash
# Query for relevant messages based on semantic similarity
curl -X POST http://localhost:3000/query/llm-context \
  -H "Content-Type: application/json" \
  -d '{
    "query": "How do I install a Python package?",
    "top_k": 5,
    "max_tokens": 2000,
    "include_kg_edges": true
  }' | jq
```

**Example Response:**
```json
{
  "formatted_context": {
    "messages": [
      {
        "role": "user",
        "content": "pip install editdistance",
        "message_id": "41389ec1-cc3e-44d5-8008-bfa94abd9954",
        "relevance_score": 0.652
      },
      {
        "role": "user",
        "content": "It looks like you're trying to import selenium...",
        "message_id": "...",
        "relevance_score": 0.421
      }
    ],
    "total_tokens_estimate": 150,
    "context_window_used": 7.5,
    "unique_conversations": 2
  },
  "knowledge_graph_edges": [
    {
      "source": "user",
      "target": "install_package",
      "relation": "wants_to",
      "evidence_message_ids": ["..."],
      "conversation_id": "..."
    }
  ],
  "query_duration_ms": 127,
  "total_evidence_messages": 5
}
```

### Getting Statistics

```bash
curl http://localhost:3000/ingest/statistics | jq
```

**Response:**
```json
{
  "total_conversations": 270,
  "total_messages": 5741,
  "total_nodes": 1768,
  "total_edges": 1561
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

### Message Storage (ag_catalog schema)

```sql
-- Conversations
CREATE TABLE conversations (
    conversation_id UUID PRIMARY KEY,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Messages
CREATE TABLE ag_catalog.messages (
    message_id UUID PRIMARY KEY,
    conversation_id UUID NOT NULL REFERENCES conversations(conversation_id),
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Message embeddings (pgvector)
CREATE TABLE ag_catalog.message_embeddings (
    message_id UUID PRIMARY KEY REFERENCES messages(message_id),
    embedding vector(768) NOT NULL,
    embedding_model VARCHAR(100) DEFAULT 'nomic-embed-text-v1.5',
    created_at TIMESTAMP DEFAULT NOW()
);

-- IVFFlat index for fast similarity search
CREATE INDEX idx_message_embeddings_ivfflat 
    ON ag_catalog.message_embeddings
    USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
```

### Knowledge Graph Storage

```sql
-- KG Nodes
CREATE TABLE kg_nodes (
    node_id VARCHAR(255),
    conversation_id UUID REFERENCES conversations(conversation_id),
    node_type VARCHAR(100),
    created_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY (node_id, conversation_id)
);

-- KG Edges with evidence tracking
CREATE TABLE kg_edges (
    edge_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID REFERENCES conversations(conversation_id),
    source_node VARCHAR(255) NOT NULL,
    target_node VARCHAR(255) NOT NULL,
    relation VARCHAR(255) NOT NULL,
    evidence_message_ids UUID[] NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_kg_edges_evidence ON kg_edges USING GIN(evidence_message_ids);
```

### Legacy Edge Embeddings (ag_catalog schema)

```sql
-- LSH-indexed edge embeddings
CREATE TABLE ag_catalog.embeddings (
    triplet_id BIGINT PRIMARY KEY,
    vec TEXT,
    lsh_bucket INTEGER,
    session_id TEXT,
    edge_text TEXT
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

### Completed ✅
- [x] REST API endpoints for ingestion/retrieval
- [x] Semantic embeddings with llama.cpp (Nomic Embed v1.5)
- [x] Message-level RAG with pgvector
- [x] Conversation-aware knowledge graphs
- [x] Evidence tracking linking messages to KG edges
- [x] LLM context generation with relevance scoring
- [x] Token budget management for context windows
- [x] IVFFlat indexing for fast similarity search

### In Progress 🚧
- [ ] Hybrid retrieval (semantic + keyword + graph traversal)
- [ ] Query result caching
- [ ] Batch embedding generation optimization

### Future Plans 📋
- [ ] Support for multiple embedding models (OpenAI, Cohere, etc.)
- [ ] Streaming ingestion API for real-time updates
- [ ] Re-ranking with cross-encoders
- [ ] Conversation summarization
- [ ] Multi-turn conversation context
- [ ] Monitoring and metrics dashboard (Prometheus/Grafana)
- [ ] Docker containerization
- [ ] Kubernetes deployment manifests
- [ ] Distributed deployment for large-scale workloads

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
- **[PostgreSQL](https://www.postgresql.org/)** - Advanced open-source relational database
- **[pgvector](https://github.com/pgvector/pgvector)** - Vector similarity search extension for PostgreSQL
- **[Apache AGE](https://age.apache.org/)** - Graph database extension for PostgreSQL
- **[llama.cpp](https://github.com/ggerganov/llama.cpp)** - Efficient LLM inference in C++
- **[Nomic Embed](https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF)** - State-of-the-art text embedding model (768-dim)
- **[Tokio](https://tokio.rs/)** - Async runtime for Rust
- **[Axum](https://github.com/tokio-rs/axum)** - Web framework for Rust
- **[tokio-postgres](https://github.com/sfackler/rust-postgres)** - PostgreSQL client for Rust

## Performance Characteristics

### Ingestion
- **Message Ingestion**: 5,741 messages in 3.5 seconds (~1,640 messages/sec)
- **Knowledge Graph Ingestion**: 3,329 nodes+edges in 963ms (~3,455 items/sec)
- **Embedding Storage**: Native pgvector format, no serialization overhead
- **Memory Usage**: ~2GB for llama.cpp server with Q4_0 model

### Query Performance
- **Semantic Search**: <200ms for top-10 from 5,741 messages (pgvector cosine similarity)
- **LLM Context Generation**: ~127ms end-to-end (embedding generation + retrieval + formatting)
- **Embedding Generation**: ~128ms per query via llama.cpp HTTP
- **Accuracy**: >60% semantic relevance for related queries, exact matches for direct keywords

### Scalability
- Tested with **5,741 messages** across **270 conversations**
- IVFFlat indexing provides sub-linear scaling for large datasets
- Async Rust architecture handles concurrent requests efficiently

## Support

For issues, questions, or contributions:
- Open an issue on the [GitHub repository](https://github.com/NiharR007/RustIngester)
- Check existing documentation in the repo

---

**Built with ❤️ using Rust, Apache AGE, and llama.cpp**
