# Examples

This directory contains example scripts and usage demonstrations for the **LLM Web Agent with Llama + MCP support**.

## 🐳 **Docker-Based Setup**

The web agent now uses **Docker** for simplified deployment with **Llama models** and **Model Context Protocol (MCP)** integration.

### **Key Improvements:**
- ✅ **Containerized deployment** with `make docker-up`
- ✅ **Automatic model initialization** with Llama support
- ✅ **Smart content extraction** using specialized MCP tools
- ✅ **Platform detection** (Amazon, Shopify, WooCommerce, etc.)
- ✅ **JSON-LD structured data** parsing
- ✅ **Multi-step reasoning** with tool-assisted workflows
- ✅ **Token efficiency** - no more raw HTML dumps

## Quick Start

### 1. **Prerequisites**
```bash
# Required tools
docker --version          # Docker Engine
docker-compose --version  # Docker Compose
make --version            # Make utility
jq --version              # JSON processor
```

### 2. **Start Services**
```bash
# Start everything with one command
make docker-up

# Initialize Llama models (includes llama3.2:latest)
make init-models
```

### 3. **Run Enhanced Demo**
```bash
# Run the full Llama + MCP demonstration
./examples/simple_demo.sh

# Or test individual features
./examples/demo_product_extraction.sh
```

### 4. **Check Status**
```bash
# View service status
make status

# Check application health
make health-app

# View logs
make docker-logs
```

## 🦙 **Llama + MCP Architecture**

### **MCP Tools Available**
The web agent exposes specialized tools via Model Context Protocol:

| Tool | Description | Use Case |
|------|-------------|----------|
| `extract_clean_text` | Remove HTML clutter, extract readable content | Content preprocessing |
| `extract_product_data` | CSS selectors + JSON-LD structured data extraction | E-commerce product info |
| `extract_by_selectors` | Custom CSS selector-based extraction | Targeted data extraction |
| `analyze_page_structure` | Detect platform type and suggest extraction strategy | Smart platform handling |

### **Workflow Example**
```bash
# 1. Llama receives extraction request
# 2. Analyzes page structure using MCP tools
# 3. Selects appropriate extraction strategy
# 4. Uses specialized tools for clean data extraction
# 5. Returns structured product information
```

## Example Scripts

### **🚀 `simple_demo.sh`**
**Main demonstration script** showcasing Docker + Llama + MCP integration.

Features:
- **Docker service management** with health checks
- **Multi-site product extraction** (Amazon, Tesla, Nike)
- **MCP tool discovery** and usage tracking
- **Performance metrics** and efficiency comparison

```bash
./examples/simple_demo.sh
```

### **📦 `demo_product_extraction.sh`**
**Advanced product extraction** with multiple test cases.

Features:
- **Platform detection** across different e-commerce sites
- **Structured data extraction** from JSON-LD and microdata
- **Error handling** and fallback strategies
- **Batch processing** of multiple products

### **📋 `curl_examples.md`**
**API reference** with curl examples for all endpoints.

Includes:
- **Session management** (create, navigate, extract, cleanup)
- **MCP tool usage** examples
- **Error handling** patterns
- **Advanced automation** workflows

## 🎯 **What Makes This Better**

### **Before (Raw HTML Approach)**
```bash
# Problems:
❌ 8000+ chars of messy HTML sent to LLM
❌ Include navigation, ads, scripts, styling
❌ Inefficient token usage
❌ Fragile JSON parsing
❌ No platform-specific handling
```

### **After (Llama + MCP Approach)**
```bash
# Improvements:
✅ Clean, structured data extraction
✅ Platform-aware processing
✅ Token-efficient prompts
✅ Robust error handling
✅ Multi-tool reasoning workflows
```

## Usage Patterns

### **Basic Product Extraction**
```bash
# Start services
make docker-up

# Extract product information with just a URL
curl -X POST "http://localhost:3000/product/information" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.amazon.com/dp/B08N5WRWNW"}'

# That's it! No session management needed.
```

### **Advanced Session Management (Optional)**
For advanced users who need more control:
```bash
# Create session manually
SESSION_ID=$(curl -s -X POST "http://localhost:3000/browser/session" \
  -H "Content-Type: application/json" \
  -d '{"timeout_seconds": 60}' | jq -r '.session_id')

# Navigate to product page
curl -X POST "http://localhost:3000/browser/navigate" \
  -H "Content-Type: application/json" \
  -d "{\"session_id\": \"$SESSION_ID\", \"url\": \"https://www.amazon.com/dp/B08N5WRWNW\"}"

# Extract with custom selector
curl -X POST "http://localhost:3000/browser/extract" \
  -H "Content-Type: application/json" \
  -d "{\"session_id\": \"$SESSION_ID\", \"selector\": \"body\"}"
```

### **Advanced Batch Processing**
```bash
# Process multiple products
for url in "https://shop.tesla.com/product/cybertruck-basecamp-tent" \
           "https://www.nike.com/t/air-max-90-mens-shoes-6n7J06/CN8490-002"; do
  echo "Processing: $url"
  # ... extraction logic
done
```

## Troubleshooting

### **Common Issues**

**🔧 Services won't start**
```bash
# Clean and restart
make docker-down
make docker-clean
make docker-up
```

**🦙 Models not available**
```bash
# Reinitialize models
make init-models

# Check model status
docker exec mistral-local ollama list
```

**🌐 Extraction failures**
```bash
# Check service logs
make docker-logs

# Test health endpoints
make health-app
curl http://localhost:3000/.well-known/mcp/manifest.json
```

**💾 Storage issues**
```bash
# Clean up volumes
make docker-clean

# Check disk space
docker system df
```

## Performance

### **Benchmarks**
- **Setup time**: ~30-60 seconds (including model download)
- **Extraction speed**: ~2-5 seconds per product page
- **Memory usage**: ~1-2GB (includes Ollama + models)
- **Token efficiency**: ~90% reduction vs raw HTML

### **Load Testing**
```bash
# Light load test
make load-test-light

# Heavy stress test  
make load-test-heavy

# Full workflow test
make load-test-workflow
```

## Development

### **Adding New MCP Tools**
1. Add tool definition to `src/mcp_server.rs`
2. Implement tool logic with HTML/CSS processing
3. Update manifest at `/.well-known/mcp/manifest.json`
4. Test with `./examples/simple_demo.sh`

### **Customizing Llama Models**
```bash
# Add new models to init-models in Makefile
docker exec mistral-local ollama pull llama3.3:latest

# Update environment variables in docker-compose.yml
# LLAMA_MODEL=llama3.3:latest
```

---

🎉 **Ready to extract product data at scale with Llama + MCP + Docker!** 