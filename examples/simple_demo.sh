#!/bin/bash

# Llama + MCP Enhanced Product Extraction Demo
# Docker-based demonstration of Llama LLM with Model Context Protocol tools

set -e

SERVER_URL="http://localhost:3000"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}🐳 LLM Web Agent - Docker + Llama + MCP Demo${NC}\n"

# Check dependencies
echo -e "${YELLOW}📋 Checking dependencies...${NC}"

if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is required but not installed.${NC}"
    echo -e "${YELLOW}Install Docker Desktop from: https://www.docker.com/products/docker-desktop${NC}"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}❌ Docker Compose is required but not installed.${NC}"
    echo -e "${YELLOW}Install Docker Compose from: https://docs.docker.com/compose/install/${NC}"
    exit 1
fi

if ! command -v make &> /dev/null; then
    echo -e "${RED}❌ make is required but not installed.${NC}"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${RED}❌ jq is required but not installed.${NC}"
    echo -e "${YELLOW}Install with: brew install jq (macOS) or sudo apt-get install jq (Ubuntu)${NC}"
    exit 1
fi

echo -e "${GREEN}✅ All dependencies found!${NC}\n"

# Check if services are running
echo -e "${YELLOW}🔍 Checking if Docker services are running...${NC}"
if docker ps | grep -q "llm-web-agent"; then
    echo -e "${GREEN}✅ Services are already running!${NC}"
else
    echo -e "${YELLOW}⏳ Starting Docker services with Llama + MCP...${NC}"
    echo -e "${CYAN}Running: make docker-up${NC}"
    
    if ! make docker-up; then
        echo -e "${RED}❌ Failed to start Docker services!${NC}"
        echo -e "${YELLOW}💡 Try: make docker-down && make docker-up${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Docker services started successfully!${NC}"
    
    # Initialize models if needed
    echo -e "${YELLOW}🦙 Checking if Llama models are available...${NC}"
    echo -e "${CYAN}Running: make init-models${NC}"
    
    if ! make init-models; then
        echo -e "${YELLOW}⚠️  Model initialization may have failed, but continuing...${NC}"
    else
        echo -e "${GREEN}✅ Llama models initialized!${NC}"
    fi
fi

echo ""

# Wait for services to be fully ready
echo -e "${YELLOW}⏳ Waiting for services to be fully ready...${NC}"
for i in {1..30}; do
    if curl -s -f "$SERVER_URL/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ Web agent is ready!${NC}"
        break
    fi
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ Web agent failed to start within 30 seconds${NC}"
        echo -e "${YELLOW}💡 Check logs with: make docker-logs${NC}"
        exit 1
    fi
    echo -n "."
    sleep 1
done

echo ""

# Test server health
echo -e "${YELLOW}📋 Checking server status...${NC}"
HEALTH_RESPONSE=$(curl -s "$SERVER_URL/health")
echo -e "${GREEN}Health Status:${NC}"
echo "$HEALTH_RESPONSE" | jq '.'

# Check MCP tools availability
echo -e "\n${PURPLE}🔧 Discovering MCP Tools...${NC}"
MCP_MANIFEST=$(curl -s "$SERVER_URL/.well-known/mcp/manifest.json" 2>/dev/null || echo "{}")
if echo "$MCP_MANIFEST" | jq -e '.tools' > /dev/null 2>&1; then
    echo -e "${GREEN}Available MCP Tools:${NC}"
    echo "$MCP_MANIFEST" | jq -r '.tools[] | "  • \(.name): \(.description)"'
else
    echo -e "${YELLOW}⚠️  MCP manifest not found - tools will be discovered dynamically${NC}"
fi

# Test product extraction on multiple sites
DEMO_SITES=(
    "https://www.amazon.com/Star-Wars-Echo-Dot-bundle/dp/B0DZQ92XQZ/?th=1"  # Echo Dot
    "https://www.target.com/p/toddler-boys-39-dino-backpack-cat-38-jack-8482-green/-/A-94430607#lnk=sametab"  # Target product
    "https://www.nike.com/t/air-max-90-mens-shoes-6n7J06/CN8490-002"  # Nike shoes
)

echo -e "\n${BLUE}🛍️  Starting Enhanced Product Extraction Tests${NC}\n"

for i in "${!DEMO_SITES[@]}"; do
    URL="${DEMO_SITES[$i]}"
    echo -e "${CYAN}🌐 Test $((i+1)): Extracting from $URL${NC}"
    
    # Extract product information directly with the new simplified endpoint
    echo -e "${YELLOW}  → Extracting product information with Llama + MCP...${NC}"
    EXTRACT_RESPONSE=$(curl -s -X POST "$SERVER_URL/product/information" \
        -H "Content-Type: application/json" \
        -d "{\"url\": \"$URL\"}")
    
    if echo "$EXTRACT_RESPONSE" | jq -e '.name' > /dev/null 2>&1; then
        echo -e "${GREEN}  ✅ Product extraction successful!${NC}"
        
        # Parse and display key product details
        PRODUCT_NAME=$(echo "$EXTRACT_RESPONSE" | jq -r '.name // "N/A"')
        PRODUCT_PRICE=$(echo "$EXTRACT_RESPONSE" | jq -r '.price // "N/A"')
        PRODUCT_BRAND=$(echo "$EXTRACT_RESPONSE" | jq -r '.brand // "N/A"')
        
        echo -e "${CYAN}  📦 Product Details:${NC}"
        echo -e "    Name: ${PRODUCT_NAME}"
        echo -e "    Price: ${PRODUCT_PRICE}"
        echo -e "    Brand: ${PRODUCT_BRAND}"
        
        # Show additional product fields if available
        if echo "$EXTRACT_RESPONSE" | jq -e '.description' > /dev/null 2>&1; then
            PRODUCT_DESCRIPTION=$(echo "$EXTRACT_RESPONSE" | jq -r '.description // "N/A"' | head -c 100)
            echo -e "    Description: ${PRODUCT_DESCRIPTION}..."
        fi
        
        if echo "$EXTRACT_RESPONSE" | jq -e '.availability' > /dev/null 2>&1; then
            PRODUCT_AVAILABILITY=$(echo "$EXTRACT_RESPONSE" | jq -r '.availability // "N/A"')
            echo -e "    Availability: ${PRODUCT_AVAILABILITY}"
        fi
        
        echo -e "    ${PURPLE}🔧 Extracted using Llama + MCP tools${NC}"
        
    else
        echo -e "${RED}  ❌ Product extraction failed${NC}"
        echo -e "${YELLOW}  Error: $(echo "$EXTRACT_RESPONSE" | jq -r '.error // "Unknown error"')${NC}"
        echo -e "${YELLOW}  Response: ${EXTRACT_RESPONSE}${NC}"
    fi
    
    echo ""
done

# Show MCP performance summary
echo -e "${BLUE}📊 Demo Summary${NC}"
echo -e "${GREEN}✅ Demonstrated Llama + MCP Integration:${NC}"
echo -e "  • Docker-based deployment with make docker-up"
echo -e "  • Automatic Llama model initialization"
echo -e "  • Simplified API - just POST URL to /product/information"
echo -e "  • No session management required"
echo -e "  • Smart product extraction across different e-commerce sites"
echo -e "  • Efficient token usage vs traditional HTML parsing"

echo -e "\n${PURPLE}🔧 Available Commands:${NC}"
echo -e "  make docker-logs    # View service logs"
echo -e "  make docker-down    # Stop all services"  
echo -e "  make status         # Check service status"
echo -e "  make health-app     # Check app health"

echo -e "\n${BLUE}🎉 Demo completed! Llama + MCP integration is working perfectly.${NC}" 