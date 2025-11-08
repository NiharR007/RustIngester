#!/bin/bash
# Cleanup script - Remove outdated documentation and scripts

echo "╔════════════════════════════════════════════════════════════╗"
echo "║         RustIngester - Cleanup Obsolete Files             ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

# Count files before
BEFORE_COUNT=$(ls -1 *.md *.sh *.sql 2>/dev/null | wc -l)

echo "📋 Files before cleanup: $BEFORE_COUNT"
echo ""
echo "Removing outdated files..."
echo ""

# Duplicate/outdated documentation
echo "🗑️  Removing duplicate Docker documentation..."
rm -f DOCKER.md DOCKER_SUCCESS.md

echo "🗑️  Removing old implementation guides..."
rm -f IMPLEMENTATION_GUIDE.md IMPLEMENTATION_SUMMARY.md CHANGES_SUMMARY.md
rm -f FINAL_ARCHITECTURE.md HYBRID_RETRIEVAL_GUIDE.md

echo "🗑️  Removing old setup guides..."
rm -f QUICK_START.md README_SERVICE.md SETUP_DATABASE.md
rm -f EMBEDDING_SETUP.md TESTING_GUIDE.md TEST_DOCUMENTATION.md

echo "🗑️  Removing obsolete scripts..."
rm -f setup_and_test.sh fix_complete_setup.sh QUICK_TEST.sh
rm -f test_hybrid_retrieval.sh diagnose_database.sh
rm -f docker-start.sh docker-stop.sh cleanup_duplicates.sql

# Count files after
AFTER_COUNT=$(ls -1 *.md *.sh *.sql 2>/dev/null | wc -l)
REMOVED=$((BEFORE_COUNT - AFTER_COUNT))

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║                    Cleanup Complete                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Summary:"
echo "  - Files before: $BEFORE_COUNT"
echo "  - Files after:  $AFTER_COUNT"
echo "  - Removed:      $REMOVED files"
echo ""
echo "✅ Essential files remaining:"
echo ""
echo "📚 Documentation:"
echo "  ✓ README.md                      - Main documentation"
echo "  ✓ QUICKSTART_DOCKER.md           - Docker quick start"
echo "  ✓ DOCKER_DEPLOYMENT_SUCCESS.md   - Deployment guide"
echo "  ✓ COMPLETION_SUMMARY.md          - Task completion summary"
echo "  ✓ QUICK_REFERENCE.md             - Quick reference card"
echo ""
echo "🔧 Scripts:"
echo "  ✓ test_docker_setup.sh           - Docker system tests"
echo "  ✓ download-model.sh              - Download embedding model"
echo "  ✓ run_tests.sh                   - Rust test runner"
echo ""
echo "🐳 Docker:"
echo "  ✓ docker-compose.yml             - Service orchestration"
echo "  ✓ Dockerfile                     - RustIngester image"
echo "  ✓ docker/Dockerfile.postgres     - PostgreSQL with AGE"
echo "  ✓ docker/init-db.sql             - Database initialization"
echo ""
echo "🎉 Repository is now clean and ready for git push!"
echo ""
