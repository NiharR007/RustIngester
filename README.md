# RustIngester

A high-performance Rust-based knowledge graph ingestion and retrieval system using PostgreSQL with Apache AGE (A Graph Extension) and Locality Sensitive Hashing (LSH) for efficient similarity search.

## Overview

RustIngester provides a complete pipeline for:
- **Ingesting** semantic triplets (subject-relationship-object) into a graph database
- **Embedding** triplets as vectors for similarity search
- **Indexing** using LSH for fast approximate nearest neighbor retrieval
- **Querying** similar triplets using cosine similarity

## Features

- 🚀 **High Performance**: Built with Rust and async I/O using Tokio
- 📊 **Graph Database**: Apache AGE for flexible graph storage and queries
- 🔍 **Vector Search**: LSH-based similarity search with configurable buckets
- 🧪 **Comprehensive Testing**: 8 test suites covering all components
- 🔄 **Async Pipeline**: Non-blocking ingestion and retrieval operations

## Architecture

```
┌─────────────┐
│   Triplet   │
│   Input     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Extraction │
│   & Parse   │
└──────┬──────┘
       │
       ▼
┌─────────────┐      ┌──────────────┐
│  Embedding  │─────▶│ LSH Hashing  │
│ Generation  │      └──────┬───────┘
└──────┬──────┘             │
       │                    │
       ▼                    ▼
┌─────────────┐      ┌──────────────┐
│ AGE Graph   │      │   Vector     │
│   Storage   │      │   Storage    │
└─────────────┘      └──────────────┘
       │                    │
       └────────┬───────────┘
                ▼
         ┌─────────────┐
         │  Retrieval  │
         │   & Query   │
         └─────────────┘
```

## Prerequisites

- **Rust**: 1.70 or higher
- **PostgreSQL**: 14.0 or higher
- **Apache AGE**: 1.5.0 or higher
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

### 5. Configure Environment

Create a `.env` file in the project root:

```bash
# .env
DATABASE_URL=postgresql://postgres:password@localhost:5432/postgres
LSH_BUCKETS=128
```

**Configuration Parameters**:
- `DATABASE_URL`: PostgreSQL connection string
  - Format: `postgresql://[user]:[password]@[host]:[port]/[database]`
- `LSH_BUCKETS`: Number of LSH buckets for similarity search (default: 128)

### 6. Build the Project

```bash
# Install dependencies and build
cargo build --release
```

## Usage

### Running the Main Application

The main application demonstrates ingestion and retrieval:

```bash
cargo run --release
```

This will:
1. Ingest an example triplet (alice → AUTHORED_BY → email_123)
2. Query for similar triplets
3. Display results with similarity scores

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
│   ├── main.rs              # Main entry point
│   ├── lib.rs               # Library exports
│   ├── config.rs            # Configuration management
│   ├── connect_db.rs        # Database connection
│   ├── ingest.rs            # Ingestion pipeline
│   ├── retrieve.rs          # Retrieval and similarity search
│   ├── graph_ops.rs         # Graph operations
│   ├── tests.rs             # Test suite
│   ├── db/
│   │   ├── connect.rs       # Database client setup
│   │   ├── graph.rs         # AGE graph operations
│   │   ├── vector.rs        # Vector storage operations
│   │   └── mod.rs
│   └── etl/
│       ├── parser.rs        # Triplet parsing
│       ├── embed.rs         # Embedding generation
│       ├── lsh.rs           # LSH hashing
│       └── mod.rs
├── Cargo.toml               # Rust dependencies
├── .env                     # Environment configuration (create this)
├── run_tests.sh             # Test runner script
├── TEST_DOCUMENTATION.md    # Test documentation
└── README.md                # This file
```

## API Reference

### Ingestion

```rust
use rust_ingester::ingest::ingest_triplet;
use rust_ingester::etl::parser::ParsedTriplet;

let triplet = ParsedTriplet {
    id: 1,
    subject: "alice".into(),
    relationship: "AUTHORED_BY".into(),
    object: "email_123".into(),
    ..Default::default()
};

ingest_triplet(triplet).await?;
```

### Retrieval

```rust
use rust_ingester::retrieve::query_similar;

// Query for top 5 similar triplets
let results = query_similar("alice email", 5).await?;

for (triplet_id, distance) in results {
    println!("Triplet ID: {}, Distance: {}", triplet_id, distance);
}
```

## Database Schema

### AGE Graph Schema

The system creates a graph named `sem_graph` with the following structure:

- **Nodes**: Represent entities (subjects and objects)
  - Properties: `name`, `type`, custom properties
- **Edges**: Represent relationships
  - Label: Relationship type (e.g., "AUTHORED_BY")
  - Properties: Custom relationship properties

### Embeddings Table

```sql
CREATE TABLE embeddings (
    triplet_id BIGINT PRIMARY KEY,
    vec TEXT,              -- JSON-serialized vector
    lsh_bucket INTEGER     -- LSH bucket for fast lookup
);
```

## Performance Tuning

### LSH Buckets

Adjust the number of LSH buckets based on your dataset size:

```bash
# In .env file
LSH_BUCKETS=128   # Default, good for small-medium datasets
LSH_BUCKETS=256   # Better precision for larger datasets
LSH_BUCKETS=512   # High precision, more memory usage
```

**Trade-offs**:
- **More buckets**: Higher precision, more memory, slower insertion
- **Fewer buckets**: Faster insertion, less memory, lower precision

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

- [ ] Streaming ingestion API
- [ ] REST API endpoints for ingestion/retrieval
- [ ] Support for multiple embedding models
- [ ] Distributed LSH for large-scale deployments
- [ ] Real-time graph updates
- [ ] Query optimization and caching
- [ ] Monitoring and metrics

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

## Acknowledgments

- [Apache AGE](https://age.apache.org/) - Graph database extension for PostgreSQL
- [Tokio](https://tokio.rs/) - Async runtime for Rust
- [tokio-postgres](https://github.com/sfackler/rust-postgres) - PostgreSQL client for Rust

## Support

For issues, questions, or contributions, please open an issue on the GitHub repository.

---

**Built with ❤️ using Rust and Apache AGE**
