#!/bin/bash

# Simple Product Extraction Demo
# Quick demonstration of the product extraction endpoint

set -e

SERVER_URL="http://localhost:3000"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}🚀 LLM Web Agent - Simple Product Extraction Demo${NC}\n"

# Check if jq is available
if ! command -v jq &> /dev/null; then
    echo -e "${RED}❌ jq is required but not installed.${NC}"
    echo -e "${YELLOW}Install with: brew install jq (macOS) or sudo apt-get install jq (Ubuntu)${NC}"
    exit 1
fi

# Test server health
echo -e "${YELLOW}📋 Checking server status...${NC}"
if curl -s -f "$SERVER_URL/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Server is running${NC}"
    
    # Show server info
    server_info=$(curl -s "$SERVER_URL/health" | jq -r '.message')
    echo -e "   Status: $server_info"
else
    echo -e "${RED}❌ Server not running at $SERVER_URL${NC}"
    echo -e "${YELLOW}Start server with: cargo run --release${NC}"
    exit 1
fi

echo -e "\n${YELLOW}📦 Testing product extraction...${NC}"

# Simple test with httpbin (always available)
echo -e "${BLUE}Testing with basic URL...${NC}"
response=$(curl -s -X POST "$SERVER_URL/product/information" \
    -H "Content-Type: application/json" \
    -d '{"url": "https://httpbin.org/html"}')

success=$(echo "$response" | jq -r '.success // false')
if [[ "$success" == "true" ]]; then
    echo -e "${GREEN}✅ Product extraction endpoint is working!${NC}"
    extraction_time=$(echo "$response" | jq -r '.extraction_time_ms // 0')
    echo -e "   Extraction completed in ${extraction_time}ms"
    
    # Show basic product structure
    echo -e "\n${YELLOW}Product information structure:${NC}"
    echo "$response" | jq '.product | keys[]' | sort
else
    echo -e "${RED}❌ Product extraction failed${NC}"
    echo "$response" | jq '.'
fi

echo -e "\n${GREEN}🎯 Demo complete!${NC}"
echo -e "\n${YELLOW}To run the full demo:${NC} ./examples/demo_product_extraction.sh"
echo -e "${YELLOW}To test manually:${NC}"
echo -e "curl -X POST $SERVER_URL/product/information \\"
echo -e "  -H \"Content-Type: application/json\" \\"
echo -e "  -d '{\"url\": \"https://www.amazon.com/dp/B08N5WRWNW\"}'" 