# RustIngester REST API Service

> **New!** RustIngester now includes a complete REST API service for ingesting and querying knowledge graphs from the ok.json format.

## 🚀 Quick Start

### 1. Configure Database

Update `.env` file:
```bash
DATABASE_URL=postgresql://postgres:your_password@localhost:5432/postgres
LSH_BUCKETS=128
SERVER_PORT=3000
```

See [SETUP_DATABASE.md](SETUP_DATABASE.md) for detailed configuration options.

### 2. Ingest Data (CLI)

```bash
cargo run --release --bin ingest_cli -- Data/ok.json
```

### 3. Start Service

```bash
cargo run --release --bin service
```

### 4. Test API

```bash
# Health check
curl http://localhost:3000/status

# Ingest data
curl -X POST http://localhost:3000/ingest/batch \
  -H "Content-Type: application/json" \
  -d @Data/ok.json

# Query similar edges
curl -X POST http://localhost:3000/query/similar \
  -H "Content-Type: application/json" \
  -d '{"query": "user python package", "top_k": 5}'
```

## 📚 Documentation

- **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Complete implementation overview
- **[IMPLEMENTATION_GUIDE.md](IMPLEMENTATION_GUIDE.md)** - Technical architecture details
- **[TESTING_GUIDE.md](TESTING_GUIDE.md)** - Comprehensive testing instructions
- **[SETUP_DATABASE.md](SETUP_DATABASE.md)** - Database configuration guide
- **[QUICK_START.md](QUICK_START.md)** - Quick reference commands

## 🎯 API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/status` | GET | System health and statistics |
| `/ingest/session` | POST | Ingest single session graph |
| `/ingest/batch` | POST | Ingest multiple sessions |
| `/query/similar` | POST | Find similar edges by text |
| `/query/session/:id` | GET | Retrieve session graph |
| `/graph/cypher` | POST | Execute Cypher queries |

## 📦 What's New

### Data Models
- ✅ Support for ok.json format (session-based knowledge graphs)
- ✅ Evidence tracking with `evidence_message_ids`
- ✅ Node types and relationships from your data

### Batch Ingestion
- ✅ Ingest entire ok.json file at once
- ✅ Progress tracking and error reporting
- ✅ Session metadata storage

### REST API
- ✅ Full CRUD operations via HTTP
- ✅ JSON request/response
- ✅ CORS enabled
- ✅ Request tracing

### CLI Tools
- ✅ `ingest_cli` - Command-line batch ingestion
- ✅ `service` - Web service binary

## 🏗️ Architecture

```
ok.json → API/CLI → Ingestion Pipeline → PostgreSQL + AGE
                                       → Vector Storage (LSH)
                                       → Evidence Tracking
```

## 📊 Example Usage

### Ingest Session

```bash
curl -X POST http://localhost:3000/ingest/session \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "my-session-001",
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
  }'
```

### Query Similar

```bash
curl -X POST http://localhost:3000/query/similar \
  -H "Content-Type: application/json" \
  -d '{
    "query": "person learning programming language",
    "top_k": 5,
    "threshold": 0.7
  }'
```

Response:
```json
{
  "results": [
    {
      "session_id": "my-session-001",
      "edge": {
        "source": "Alice",
        "relation": "knows",
        "target": "Python"
      },
      "similarity": 0.92,
      "distance": 0.08,
      "evidence_message_ids": ["msg-001"]
    }
  ],
  "count": 1
}
```

## 🔧 Development

### Build
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Format Code
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

## 📈 Performance

With ok.json (10 sessions, ~75 nodes, ~65 edges):
- **CLI Ingestion**: 1-3 seconds
- **API Batch**: 2-4 seconds
- **Single Session**: 100-300ms
- **Query**: 50-200ms

## 🛠️ Troubleshooting

See [SETUP_DATABASE.md](SETUP_DATABASE.md) for common issues:
- Database authentication
- AGE extension installation
- Connection problems

## 🎓 Learn More

- [Apache AGE Documentation](https://age.apache.org/)
- [Axum Web Framework](https://docs.rs/axum/)
- [Original README](README.md)

---

**Built with Rust 🦀 | Powered by PostgreSQL + Apache AGE**
