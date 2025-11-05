#!/bin/bash
set -e

echo "🚀 Starting RustIngester with Docker Compose..."

# Check if models directory exists
if [ ! -d "models" ] || [ ! -f "models/nomic-embed-text-v1.5.Q4_0.gguf" ]; then
    echo "⚠️  Warning: Model file not found!"
    echo "📥 Please download the model first:"
    echo "   mkdir -p models"
    echo "   cd models"
    echo "   wget https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_0.gguf"
    echo ""
    read -p "Do you want to continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker Desktop."
    exit 1
fi

# Build and start services
echo "🔨 Building Docker images..."
docker compose build

echo "🚀 Starting services..."
docker compose up -d

echo ""
echo "⏳ Waiting for services to be healthy..."
sleep 10

# Check service health
echo ""
echo "📊 Service Status:"
docker compose ps

echo ""
echo "✅ RustIngester is starting up!"
echo ""
echo "🔍 Check status:"
echo "   curl http://localhost:3000/status"
echo ""
echo "📝 View logs:"
echo "   docker compose logs -f rustingester"
echo ""
echo "🛑 Stop services:"
echo "   docker compose down"
echo ""
