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
PURPLE='\033[0;35m'
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

# Real product extraction test
echo -e "\n${YELLOW}🛍️  Testing real product extraction...${NC}"
echo -e "${BLUE}Extracting from Amazon Star Wars Echo Dot...${NC}"

real_response=$(curl -s -X POST "$SERVER_URL/product/information" \
    -H "Content-Type: application/json" \
    -d '{"url": "https://www.amazon.com/Star-Wars-Echo-Dot-bundle/dp/B0DZQ92XQZ/?th=1"}')

real_success=$(echo "$real_response" | jq -r '.success // false')
if [[ "$real_success" == "true" ]]; then
    echo -e "${GREEN}✅ Real product extraction successful!${NC}"
    real_extraction_time=$(echo "$real_response" | jq -r '.extraction_time_ms // 0')
    echo -e "   Extraction completed in ${real_extraction_time}ms"
    
    # Show extracted real product information
    echo -e "\n${YELLOW}🎯 Parsed Product Information:${NC}"
    
    real_name=$(echo "$real_response" | jq -r '.product.name // "Not found"')
    real_description=$(echo "$real_response" | jq -r '.product.description // "Not found"')
    real_price=$(echo "$real_response" | jq -r '.product.price // "Not found"')
    real_availability=$(echo "$real_response" | jq -r '.product.availability // "Not found"')
    real_brand=$(echo "$real_response" | jq -r '.product.brand // "Not found"')
    real_rating=$(echo "$real_response" | jq -r '.product.rating // "Not found"')
    
    echo -e "   ${BLUE}Name:${NC} $real_name"
    echo -e "   ${BLUE}Description:${NC} $real_description"
    echo -e "   ${BLUE}Price:${NC} $real_price"
    echo -e "   ${BLUE}Availability:${NC} $real_availability"
    echo -e "   ${BLUE}Brand:${NC} $real_brand"
    echo -e "   ${BLUE}Rating:${NC} $real_rating"
    
    # Show raw LLM response
    echo -e "\n${PURPLE}🤖 Raw LLM Response:${NC}"
    raw_llm_response=$(echo "$real_response" | jq -r '.product.raw_llm_response // "No raw response available"')
    if [[ "$raw_llm_response" != "No raw response available" && "$raw_llm_response" != "null" ]]; then
        echo -e "${PURPLE}   $raw_llm_response${NC}"
    else
        echo -e "${YELLOW}   ⚠️  Raw LLM response not available (field: $raw_llm_response)${NC}"
        
        # As fallback, show raw_data field which contains LLM response
        raw_data=$(echo "$real_response" | jq -r '.product.raw_data // "No raw data available"')
        if [[ "$raw_data" != "No raw data available" && "$raw_data" != "null" ]]; then
            echo -e "\n${PURPLE}🔍 Raw Data (LLM Response):${NC}"
            echo -e "${PURPLE}   $raw_data${NC}"
        fi
    fi
    
else
    echo -e "${YELLOW}⚠️  Real product extraction had issues (this is normal if LLM is not available)${NC}"
    echo -e "   Error: $(echo "$real_response" | jq -r '.error // "Unknown error"')"
fi

echo -e "\n${GREEN}🎯 Demo complete!${NC}"
echo -e "\n${YELLOW}To run the full demo:${NC} ./examples/demo_product_extraction.sh"
echo -e "${YELLOW}To test manually:${NC}"
echo -e "curl -X POST $SERVER_URL/product/information \\"
echo -e "  -H \"Content-Type: application/json\" \\"
echo -e "  -d '{\"url\": \"https://www.amazon.com/dp/B08N5WRWNW\"}'" 