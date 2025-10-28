# RustIngester Test Suite Documentation

This document describes the comprehensive test suite for the RustIngester application, which tests extraction, ingestion, and retrieval functionalities.

## Test Overview

The test suite covers the complete data pipeline:

1. **Extraction** - Parsing and data structure creation
2. **Ingestion** - Graph storage, embedding generation, and vector storage  
3. **Retrieval** - Similarity search and result ranking

## Test Categories

### 1. Database Tests (`test_database_connection`, `test_vector_storage`)

**Purpose**: Verify database connectivity and vector storage functionality.

**What it tests**:
- PostgreSQL connection establishment
- Embeddings table operations
- JSON vector serialization/deserialization
- LSH bucket storage

**Key assertions**:
- Database connection succeeds
- Vector data round-trip integrity
- Proper LSH bucket assignment

### 2. AGE Graph Tests (`test_age_graph_operations`)

**Purpose**: Test Apache AGE graph database operations.

**What it tests**:
- Node creation with properties
- Edge creation between nodes
- AGE cypher query execution
- Graph ID generation

**Key assertions**:
- Nodes get unique positive IDs
- Edges connect correct nodes
- AGE labels are created properly

### 3. Extraction/Parsing Tests (`test_extraction_parsing`)

**Purpose**: Verify data extraction and JSON parsing functionality.

**What it tests**:
- JSON triplet parsing
- Property extraction
- Data structure validation
- Error handling for malformed input

**Key assertions**:
- Correct field extraction from JSON
- Property values match expected types
- Nested JSON objects parsed correctly

### 4. LSH Tests (`test_lsh_hashing`)

**Purpose**: Test Locality Sensitive Hashing implementation.

**What it tests**:
- Hash bucket generation
- Deterministic hashing behavior
- Bucket range validation
- Vector dimensionality handling

**Key assertions**:
- Hash values within expected range
- Consistent hashing for same input
- Different vectors may hash to different buckets

### 5. Ingestion Pipeline Tests (`test_ingestion_pipeline`)

**Purpose**: Test the complete ingestion workflow.

**What it tests**:
- End-to-end triplet ingestion
- Graph node/edge creation
- Embedding generation and storage
- LSH bucket assignment
- Database persistence

**Key assertions**:
- Triplet successfully stored in database
- Embeddings generated and persisted
- LSH buckets assigned correctly
- All components work together

### 6. Similarity Retrieval Tests (`test_similarity_retrieval`)

**Purpose**: Test similarity search and ranking functionality.

**What it tests**:
- Embedding-based similarity search
- LSH bucket filtering
- Cosine similarity calculation
- Result ranking and limiting

**Key assertions**:
- Similar content is found
- Results sorted by distance (ascending)
- LSH filtering works correctly
- Top-k limiting functions properly

### 7. End-to-End Integration Test (`test_end_to_end_pipeline`)

**Purpose**: Comprehensive integration test of the entire system.

**What it tests**:
- Complete data flow from ingestion to retrieval
- Cross-component integration
- Real-world usage scenario
- System reliability

**Key assertions**:
- Full pipeline executes without errors
- Ingested data can be successfully retrieved
- Similarity search finds relevant results
- System maintains data integrity

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Use the test runner script
./run_tests.sh [category]

# Examples:
./run_tests.sh db          # Database tests
./run_tests.sh age         # AGE graph tests  
./run_tests.sh ingestion   # Ingestion pipeline tests
./run_tests.sh e2e         # End-to-end integration test
```

### Run Individual Tests
```bash
cargo test test_database_connection
cargo test test_ingestion_pipeline
cargo test test_end_to_end_pipeline
```

## Test Data

Tests use dynamically generated test data to avoid conflicts:

- **Unique identifiers**: Timestamps ensure unique primary keys
- **Realistic data**: JSON structures mirror production data
- **Varied content**: Different triplets test various scenarios
- **Isolated tests**: Each test uses independent data

## Test Environment Requirements

- **PostgreSQL 14+** with AGE extension
- **Database**: `postgres` database accessible
- **Credentials**: Configured in `.env` file
- **Network**: Local database connection on port 5432

## Test Output

Successful test run shows:
```
running 8 tests
test tests::tests::test_database_connection ... ok
test tests::tests::test_vector_storage ... ok
test tests::tests::test_age_graph_operations ... ok
test tests::tests::test_extraction_parsing ... ok
test tests::tests::test_lsh_hashing ... ok
test tests::tests::test_ingestion_pipeline ... ok
test tests::tests::test_similarity_retrieval ... ok
test tests::tests::test_end_to_end_pipeline ... ok

test result: ok. 8 passed; 0 failed
```

## Debugging Failed Tests

### Common Issues:

1. **Database Connection Failures**
   - Check PostgreSQL is running
   - Verify credentials in `.env`
   - Ensure database exists

2. **AGE Extension Issues**
   - Verify AGE extension is installed
   - Check extension is loaded in database
   - Ensure proper search_path configuration

3. **Label Creation Conflicts**
   - Tests create unique labels automatically
   - Check for existing conflicting labels
   - Clear test data if needed

4. **Embedding API Issues**
   - Verify embedding service is accessible
   - Check API credentials if required
   - Ensure network connectivity

## Performance Considerations

- Tests run in parallel by default
- Each test uses isolated data to prevent conflicts
- Database operations are optimized for test speed
- Large vector operations are kept minimal for fast execution

## Extending Tests

To add new tests:

1. Add test function to `src/tests.rs`
2. Use `#[tokio::test]` for async tests
3. Follow naming convention: `test_[functionality]`
4. Include comprehensive assertions
5. Use unique test data to avoid conflicts
6. Add documentation for new test category

## Test Coverage

The test suite provides comprehensive coverage of:

- ✅ Database connectivity and operations
- ✅ AGE graph database functionality  
- ✅ Data extraction and parsing
- ✅ Vector embedding operations
- ✅ LSH hashing and bucketing
- ✅ Similarity search algorithms
- ✅ End-to-end system integration
- ✅ Error handling and edge cases

This ensures the RustIngester system is thoroughly validated and reliable for production use.
