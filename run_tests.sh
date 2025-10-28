#!/bin/bash

# Test runner script for RustIngester

echo "🧪 RustIngester Test Suite"
echo "=========================="

case "${1:-all}" in
    "db"|"database")
        echo "Running database tests..."
        cargo test test_database_connection test_vector_storage
        ;;
    "age"|"graph")
        echo "Running AGE graph tests..."
        cargo test test_age_graph_operations
        ;;
    "extraction"|"parsing")
        echo "Running extraction/parsing tests..."
        cargo test test_extraction_parsing
        ;;
    "ingestion")
        echo "Running ingestion pipeline tests..."
        cargo test test_ingestion_pipeline
        ;;
    "retrieval"|"similarity")
        echo "Running similarity retrieval tests..."
        cargo test test_similarity_retrieval
        ;;
    "lsh")
        echo "Running LSH tests..."
        cargo test test_lsh_hashing
        ;;
    "e2e"|"integration")
        echo "Running end-to-end integration test..."
        cargo test test_end_to_end_pipeline
        ;;
    "all")
        echo "Running all tests..."
        cargo test
        ;;
    *)
        echo "Usage: $0 [test_category]"
        echo ""
        echo "Available test categories:"
        echo "  db/database    - Database connection and vector storage tests"
        echo "  age/graph      - AGE graph operations tests"
        echo "  extraction     - Data extraction and parsing tests"
        echo "  ingestion      - Ingestion pipeline tests"
        echo "  retrieval      - Similarity retrieval tests"
        echo "  lsh            - LSH hashing tests"
        echo "  e2e/integration - End-to-end integration test"
        echo "  all            - Run all tests (default)"
        ;;
esac
