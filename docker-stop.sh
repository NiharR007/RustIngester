#!/bin/bash
set -e

echo "🛑 Stopping RustIngester services..."

docker compose down

echo "✅ All services stopped."
echo ""
echo "💡 To remove all data (including database), run:"
echo "   docker compose down -v"
echo ""
