#!/bin/bash

# Model initialization script for Ollama
set -e

OLLAMA_URL="http://mistral-local:11434"
MODELS=("mistral:7b" "mistral:latest")

echo "Waiting for Ollama service to be ready..."

# Wait for Ollama to be healthy
for i in {1..30}; do
    if curl -f "$OLLAMA_URL/api/tags" &>/dev/null; then
        echo "Ollama service is ready!"
        break
    fi
    echo "Waiting for Ollama... ($i/30)"
    sleep 10
done

if ! curl -f "$OLLAMA_URL/api/tags" &>/dev/null; then
    echo "Error: Ollama service not responding after 5 minutes"
    exit 1
fi

# Check if models are already installed
for model in "${MODELS[@]}"; do
    echo "Checking if model $model is installed..."
    
    # Check if model exists
    if curl -s "$OLLAMA_URL/api/tags" | grep -q "\"name\":\"$model\""; then
        echo "Model $model is already installed, skipping..."
        continue
    fi
    
    echo "Downloading model $model..."
    
    # Pull the model
    curl -X POST "$OLLAMA_URL/api/pull" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"$model\"}" &
    
    # Store the PID to wait for completion
    PULL_PID=$!
    
    # Wait for model download to complete
    echo "Waiting for $model to download..."
    wait $PULL_PID
    
    if [ $? -eq 0 ]; then
        echo "Successfully downloaded $model"
    else
        echo "Failed to download $model"
        exit 1
    fi
done

echo "All models initialized successfully!"

# Test the models
echo "Testing model availability..."
for model in "${MODELS[@]}"; do
    echo "Testing $model..."
    response=$(curl -s -X POST "$OLLAMA_URL/api/generate" \
        -H "Content-Type: application/json" \
        -d "{\"model\":\"$model\",\"prompt\":\"Hello\",\"stream\":false}")
    
    if echo "$response" | grep -q "response"; then
        echo "✓ $model is working correctly"
    else
        echo "✗ $model test failed"
        echo "Response: $response"
    fi
done

echo "Model initialization complete!" 