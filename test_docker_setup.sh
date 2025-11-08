#!/bin/bash

# Test script for Docker setup with Apache AGE
# Tests: PostgreSQL with AGE, llama.cpp, and RustIngester service

set -e

echo "=========================================="
echo "RustIngester Docker Setup Test"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function for tests
test_step() {
    echo -e "${YELLOW}[TEST]${NC} $1"
}

test_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((TESTS_PASSED++))
}

test_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

# 1. Check if Docker is running
test_step "Checking if Docker is running..."
if docker info > /dev/null 2>&1; then
    test_pass "Docker is running"
else
    test_fail "Docker is not running. Please start Docker Desktop."
    exit 1
fi

# 2. Check if model exists
test_step "Checking if embedding model exists..."
if [ -f "models/nomic-embed-text-v1.5.Q4_0.gguf" ]; then
    test_pass "Embedding model found"
else
    test_fail "Embedding model not found. Run: ./download-model.sh"
    exit 1
fi

# 3. Build and start services
test_step "Building and starting Docker services..."
echo "This may take 5-10 minutes on first run (building PostgreSQL with AGE)..."
if docker compose up -d --build > /tmp/docker-build.log 2>&1; then
    test_pass "Docker services started"
else
    test_fail "Failed to start Docker services. Check /tmp/docker-build.log"
    cat /tmp/docker-build.log
    exit 1
fi

# 4. Wait for services to be healthy
test_step "Waiting for services to be healthy (30 seconds)..."
sleep 30

# 5. Check PostgreSQL container
test_step "Checking PostgreSQL container..."
if docker ps | grep -q "rustingester-postgres"; then
    test_pass "PostgreSQL container is running"
else
    test_fail "PostgreSQL container is not running"
fi

# 6. Check llama.cpp container
test_step "Checking llama.cpp container..."
if docker ps | grep -q "rustingester-llama"; then
    test_pass "llama.cpp container is running"
else
    test_fail "llama.cpp container is not running"
fi

# 7. Check RustIngester container
test_step "Checking RustIngester container..."
if docker ps | grep -q "rustingester-service"; then
    test_pass "RustIngester container is running"
else
    test_fail "RustIngester container is not running"
fi

# 8. Test PostgreSQL extensions
test_step "Testing PostgreSQL extensions (pgvector, AGE)..."
EXTENSIONS=$(docker exec rustingester-postgres psql -U postgres -d rustingester -t -c "SELECT extname FROM pg_extension WHERE extname IN ('vector', 'age', 'uuid-ossp');" 2>/dev/null | tr -d ' ')

if echo "$EXTENSIONS" | grep -q "vector"; then
    test_pass "pgvector extension loaded"
else
    test_fail "pgvector extension not loaded"
fi

if echo "$EXTENSIONS" | grep -q "age"; then
    test_pass "Apache AGE extension loaded"
else
    test_fail "Apache AGE extension not loaded"
fi

# 9. Test AGE graph creation
test_step "Testing AGE graph creation..."
GRAPH_EXISTS=$(docker exec rustingester-postgres psql -U postgres -d rustingester -t -c "SELECT name FROM ag_catalog.ag_graph WHERE name = 'sem_graph';" 2>/dev/null | tr -d ' ')

if [ "$GRAPH_EXISTS" = "sem_graph" ]; then
    test_pass "AGE graph 'sem_graph' exists"
else
    test_fail "AGE graph 'sem_graph' not found"
fi

# 10. Test RustIngester API status
test_step "Testing RustIngester API status endpoint..."
STATUS_RESPONSE=$(curl -s http://localhost:3000/status)

if echo "$STATUS_RESPONSE" | jq -e '.status == "healthy"' > /dev/null 2>&1; then
    test_pass "API status is healthy"
else
    test_fail "API status is not healthy"
    echo "Response: $STATUS_RESPONSE"
fi

if echo "$STATUS_RESPONSE" | jq -e '.database == "connected"' > /dev/null 2>&1; then
    test_pass "Database connection is working"
else
    test_fail "Database connection failed"
fi

if echo "$STATUS_RESPONSE" | jq -e '.age_extension == "loaded"' > /dev/null 2>&1; then
    test_pass "AGE extension is loaded in API"
else
    test_fail "AGE extension not loaded in API"
fi

# 11. Test llama.cpp embedding server
test_step "Testing llama.cpp embedding server..."
EMBED_RESPONSE=$(curl -s -X POST http://localhost:8080/embedding \
    -H "Content-Type: application/json" \
    -d '{"content": "test"}' 2>/dev/null)

if echo "$EMBED_RESPONSE" | jq -e '.embedding | length == 768' > /dev/null 2>&1; then
    test_pass "llama.cpp embedding server is working (768-dim vectors)"
else
    test_fail "llama.cpp embedding server failed"
    echo "Response: $EMBED_RESPONSE"
fi

# 12. Test hybrid retrieval (if data exists)
if [ -f "Data/turn_embeddings.json" ]; then
    test_step "Testing message ingestion..."
    INGEST_RESPONSE=$(curl -s -X POST http://localhost:3000/ingest/messages \
        -H "Content-Type: application/json" \
        -d @Data/turn_embeddings.json)
    
    if echo "$INGEST_RESPONSE" | jq -e '.success == true' > /dev/null 2>&1; then
        test_pass "Message ingestion successful"
        
        # Test BM25 search
        test_step "Testing BM25 keyword search..."
        SEARCH_RESPONSE=$(curl -s -X POST http://localhost:3000/query/llm-context \
            -H "Content-Type: application/json" \
            -d '{"query": "Zapier", "top_k": 5, "retrieval_mode": "direct_only"}')
        
        MESSAGE_COUNT=$(echo "$SEARCH_RESPONSE" | jq -r '.retrieval_stats.direct_message_matches')
        if [ "$MESSAGE_COUNT" -gt 0 ]; then
            test_pass "BM25 search returned $MESSAGE_COUNT messages"
        else
            test_fail "BM25 search returned no messages"
        fi
    else
        test_fail "Message ingestion failed"
    fi
else
    echo -e "${YELLOW}[SKIP]${NC} Data files not found, skipping ingestion tests"
fi

# Summary
echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Failed: $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed! Docker setup is working correctly.${NC}"
    echo ""
    echo "Next steps:"
    echo "  - View logs: docker compose logs -f"
    echo "  - Test API: curl http://localhost:3000/status | jq"
    echo "  - Stop services: docker compose down"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Check the output above.${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  - View logs: docker compose logs"
    echo "  - Restart: docker compose restart"
    echo "  - Reset: docker compose down -v && docker compose up -d --build"
    exit 1
fi
