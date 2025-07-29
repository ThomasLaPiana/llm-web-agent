#!/bin/bash

# Product Information Extraction Demo Script
# This script demonstrates the LLM Web Agent's product extraction capabilities

set -e  # Exit on any error

# Configuration
SERVER_URL="http://localhost:3000"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "\n${BLUE}===================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================================${NC}\n"
}

print_step() {
    echo -e "${CYAN}📋 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check if server is running
check_server() {
    print_step "Checking if LLM Web Agent server is running..."
    
    if curl -s -f "$SERVER_URL/health" > /dev/null 2>&1; then
        print_success "Server is running at $SERVER_URL"
        
        # Get server status
        server_info=$(curl -s "$SERVER_URL/health" | jq -r '.message + " (Active sessions: " + (.active_sessions | tostring) + ")"')
        echo -e "   ${PURPLE}Status: $server_info${NC}"
    else
        print_error "Server is not running at $SERVER_URL"
        echo -e "\n${YELLOW}To start the server, run:${NC}"
        echo -e "   ${CYAN}cd $PROJECT_ROOT${NC}"
        echo -e "   ${CYAN}cargo run --release${NC}"
        echo -e "\n${YELLOW}Or in the background:${NC}"
        echo -e "   ${CYAN}cargo build --release && ./target/release/llm-web-agent &${NC}"
        exit 1
    fi
}

# Create a browser session
create_session() {
    print_step "Creating a new browser session..."
    
    local response=$(curl -s -X POST "$SERVER_URL/browser/session")
    local session_id=$(echo "$response" | jq -r '.session_id // empty')
    
    if [[ -n "$session_id" ]]; then
        print_success "Created session: $session_id"
        echo "$session_id"
    else
        print_error "Failed to create session"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        return 1
    fi
}

# Extract product information
extract_product() {
    local url="$1"
    local session_id="$2"
    local description="$3"
    
    print_step "$description"
    echo -e "   ${PURPLE}URL: $url${NC}"
    
    # Build request payload
    local payload
    if [[ -n "$session_id" ]]; then
        payload=$(jq -n --arg url "$url" --arg session_id "$session_id" '{url: $url, session_id: $session_id}')
        echo -e "   ${PURPLE}Using session: $session_id${NC}"
    else
        payload=$(jq -n --arg url "$url" '{url: $url}')
        echo -e "   ${PURPLE}Creating temporary session${NC}"
    fi
    
    # Make the request
    local start_time=$(date +%s%3N)
    local response=$(curl -s -X POST "$SERVER_URL/product/information" \
        -H "Content-Type: application/json" \
        -d "$payload")
    local end_time=$(date +%s%3N)
    local request_duration=$((end_time - start_time))
    
    # Parse response
    local success=$(echo "$response" | jq -r '.success // false')
    local extraction_time=$(echo "$response" | jq -r '.extraction_time_ms // 0')
    
    if [[ "$success" == "true" ]]; then
        print_success "Product extraction completed in ${extraction_time}ms (request: ${request_duration}ms)"
        
        # Display extracted product information
        echo -e "\n${YELLOW}📦 EXTRACTED PRODUCT INFORMATION:${NC}"
        echo -e "${CYAN}▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔${NC}"
        
        local product=$(echo "$response" | jq '.product')
        
        # Extract individual fields
        local name=$(echo "$product" | jq -r '.name // "N/A"')
        local description=$(echo "$product" | jq -r '.description // "N/A"')
        local price=$(echo "$product" | jq -r '.price // "N/A"')
        local availability=$(echo "$product" | jq -r '.availability // "N/A"')
        local brand=$(echo "$product" | jq -r '.brand // "N/A"')
        local rating=$(echo "$product" | jq -r '.rating // "N/A"')
        local image_url=$(echo "$product" | jq -r '.image_url // "N/A"')
        
        # Display formatted results
        echo -e "${GREEN}Name:${NC}         $name"
        echo -e "${GREEN}Description:${NC}  $description"
        echo -e "${GREEN}Price:${NC}        $price"
        echo -e "${GREEN}Availability:${NC} $availability"
        echo -e "${GREEN}Brand:${NC}        $brand"
        echo -e "${GREEN}Rating:${NC}       $rating"
        echo -e "${GREEN}Image URL:${NC}    $image_url"
        
        echo -e "${CYAN}▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔${NC}\n"
        
        return 0
    else
        print_error "Product extraction failed"
        local error_msg=$(echo "$response" | jq -r '.error // "Unknown error"')
        echo -e "   ${RED}Error: $error_msg${NC}"
        
        # Show raw response for debugging
        echo -e "\n${YELLOW}Raw response:${NC}"
        echo "$response" | jq '.' 2>/dev/null || echo "$response"
        echo ""
        
        return 1
    fi
}

# Main demonstration
main() {
    print_header "🚀 LLM Web Agent - Product Extraction Demo"
    
    echo -e "${PURPLE}This script demonstrates the product information extraction capabilities${NC}"
    echo -e "${PURPLE}of the LLM Web Agent using various e-commerce URLs.${NC}\n"
    
    # Check dependencies
    if ! command -v jq &> /dev/null; then
        print_error "jq is required but not installed. Please install jq first."
        echo -e "\n${YELLOW}Install jq:${NC}"
        echo -e "   ${CYAN}# On macOS:${NC} brew install jq"
        echo -e "   ${CYAN}# On Ubuntu:${NC} sudo apt-get install jq"
        exit 1
    fi
    
    # Check server status
    check_server
    
    print_header "🔍 Demonstration Scenarios"
    
    # Scenario 1: Extract without existing session (Amazon Star Wars Echo Dot)
    echo -e "${YELLOW}Scenario 1: Extract product info without existing session${NC}"
    extract_product \
        "" \
        "" \
        "Extracting Amazon Star Wars Echo Dot information (temporary session)"
    
    echo -e "\n${CYAN}────────────────────────────────────────────────────────────────${NC}\n"
    
    # Scenario 2: Create session and reuse it
    echo -e "${YELLOW}Scenario 2: Extract product info with persistent session${NC}"
    session_id=$(create_session)
    
    if [[ -n "$session_id" ]]; then
        # Extract from a different Amazon product
        extract_product \
            "https://www.amazon.com/dp/B08N5WRWNW" \
            "$session_id" \
            "Extracting Echo Dot (4th Gen) information (reusing session)"
        
        echo -e "\n${CYAN}────────────────────────────────────────────────────────────────${NC}\n"
        
        # Extract from another site
        extract_product \
            "https://httpbin.org/html" \
            "$session_id" \
            "Extracting from test page (reusing session)"
    fi
    
    print_header "🎯 Demo Complete"
    
    echo -e "${GREEN}The demonstration has completed successfully!${NC}\n"
    
    echo -e "${YELLOW}Key Features Demonstrated:${NC}"
    echo -e "  ${CYAN}•${NC} Automatic session management (temporary vs persistent)"
    echo -e "  ${CYAN}•${NC} Real-world e-commerce product extraction"
    echo -e "  ${CYAN}•${NC} Structured product information output"
    echo -e "  ${CYAN}•${NC} Error handling and debugging information"
    echo -e "  ${CYAN}•${NC} Performance timing for optimization"
    
    echo -e "\n${YELLOW}API Usage Examples:${NC}"
    echo -e "\n${CYAN}# Extract without session (creates temporary session)${NC}"
    echo -e '${PURPLE}curl -X POST http://localhost:3000/product/information \'
    echo -e '  -H "Content-Type: application/json" \'
    echo -e '  -d '"'"'{"url": "https://www.amazon.com/dp/B08N5WRWNW"}'"'"'${NC}'
    
    echo -e "\n${CYAN}# Extract with existing session${NC}"
    echo -e '${PURPLE}curl -X POST http://localhost:3000/product/information \'
    echo -e '  -H "Content-Type: application/json" \'
    echo -e '  -d '"'"'{"url": "https://www.amazon.com/dp/B08N5WRWNW", "session_id": "your-session-id"}'"'"'${NC}'
    
    echo -e "\n${YELLOW}For more information, check the documentation or run:${NC}"
    echo -e "  ${CYAN}cargo test --test product_extraction -- --nocapture${NC}\n"
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Product Extraction Demo Script"
        echo ""
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --help, -h     Show this help message"
        echo "  --check        Only check if server is running"
        echo ""
        echo "This script demonstrates the LLM Web Agent's product extraction"
        echo "capabilities by testing various e-commerce URLs."
        exit 0
        ;;
    --check)
        check_server
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac 