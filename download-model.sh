#!/bin/bash
set -e

MODEL_DIR="models"
MODEL_FILE="nomic-embed-text-v1.5.Q4_0.gguf"
MODEL_URL="https://huggingface.co/nomic-ai/nomic-embed-text-v1.5-GGUF/resolve/main/nomic-embed-text-v1.5.Q4_0.gguf"

echo "📥 Downloading Nomic Embed model..."

# Create models directory
mkdir -p "$MODEL_DIR"

# Check if model already exists
if [ -f "$MODEL_DIR/$MODEL_FILE" ]; then
    echo "✅ Model already exists: $MODEL_DIR/$MODEL_FILE"
    SIZE=$(ls -lh "$MODEL_DIR/$MODEL_FILE" | awk '{print $5}')
    echo "   Size: $SIZE"
    read -p "Do you want to re-download? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
    rm "$MODEL_DIR/$MODEL_FILE"
fi

# Download model
echo "⬇️  Downloading from HuggingFace..."
echo "   This may take a few minutes (~74 MB)..."

if command -v wget &> /dev/null; then
    wget -O "$MODEL_DIR/$MODEL_FILE" "$MODEL_URL" --progress=bar:force
elif command -v curl &> /dev/null; then
    curl -L -o "$MODEL_DIR/$MODEL_FILE" "$MODEL_URL" --progress-bar
else
    echo "❌ Error: Neither wget nor curl is installed."
    echo "   Please install one of them and try again."
    exit 1
fi

# Verify download
if [ -f "$MODEL_DIR/$MODEL_FILE" ]; then
    SIZE=$(ls -lh "$MODEL_DIR/$MODEL_FILE" | awk '{print $5}')
    echo "✅ Model downloaded successfully!"
    echo "   Location: $MODEL_DIR/$MODEL_FILE"
    echo "   Size: $SIZE"
else
    echo "❌ Download failed!"
    exit 1
fi

echo ""
echo "🚀 Ready to start! Run:"
echo "   ./docker-start.sh"
