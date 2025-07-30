# LLM Web Agent

A powerful web automation service that combines browser automation with AI-powered product extraction. This service provides a REST API for extracting product information from e-commerce websites using Llama LLM and Model Context Protocol (MCP) tools.

## Features

- **Intelligent Product Extraction**: Extract structured product data from any e-commerce website
- **Llama + MCP Integration**: Uses local Llama models with MCP tools for enhanced reasoning
- **REST API Server**: Built with Axum for high-performance web service
- **Browser Automation**: Uses chromiumoxide for headless browser control
- **Docker Deployment**: Simple containerized setup with automatic model initialization
- **Session Management**: Manages multiple browser sessions concurrently

## Quick Start

### Prerequisites

- Docker & Docker Compose
- `make` command
- `jq` command (for JSON processing)
- Chrome/Chromium browser (automatically handled in Docker)

### Setup & Usage

```bash
# Start the service
make docker-up

# Test with the demo script
./examples/simple_demo.sh

# Extract product information
curl -X POST http://localhost:3000/product/information \
  -H "Content-Type: application/json" \
  -d '{"url": "https://www.amazon.com/some-product"}'
```

## Architecture

```txt
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API      │    │   Browser       │    │   Llama + MCP   │
│   (Axum)        │◄──►│  (chromiumoxide)│◄──►│   (Ollama)      │
│                 │    │                 │    │                 │
│ • Product API   │    │ • Page Loading  │    │ • Smart Extract │
│ • Session Mgmt  │    │ • Content Get   │    │ • MCP Tools     │
│ • MCP Server    │    │ • Screenshot    │    │ • Reasoning     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## API Endpoints

### Primary Endpoints

- `GET /health` - Server health status
- `POST /product/information` - Extract product data from any e-commerce URL

### Advanced Browser Control (Optional)

- `POST /browser/session` - Create a new browser session
- `GET /browser/session/{session_id}` - Get session status
- `POST /browser/navigate` - Navigate to a URL
- `POST /browser/extract` - Extract data from current page
- `POST /automation/task` - Execute AI-planned automation tasks

### MCP Protocol

- `GET /.well-known/mcp/manifest.json` - MCP tools manifest

## Configuration

### Environment Variables

- `RUST_LOG`: Logging level (default: info)
- `PORT`: Server port (default: 3000)

### Docker Setup

```bash
# Setup and start all services
make docker-up

# View logs
make docker-logs

# Stop services  
make docker-down
```

### Additional Commands

```bash
# Check service status
make status

# Health checks
make health-app

# Load testing
make load-test-light    # Light test
make load-test-heavy    # Stress test
```

See [LOAD_TESTING.md](LOAD_TESTING.md) for detailed load testing documentation.
