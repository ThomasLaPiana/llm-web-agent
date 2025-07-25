# LLM Web Agent

A powerful web automation service that combines browser automation with AI-powered task planning. This service provides a REST API for automating web tasks using natural language descriptions.

## Features

- **REST API Server**: Built with Axum for high-performance web service
- **Browser Automation**: Uses chromiumoxide for headless browser control
- **AI Task Planning**: Integrates with Mistral LLM via API for intelligent task breakdown
- **Session Management**: Manages multiple browser sessions concurrently
- **Flexible Actions**: Supports clicking, typing, scrolling, screenshot capture, and custom JavaScript execution
- **Data Extraction**: Extract structured data from web pages using CSS selectors

## Quick Start

### Prerequisites

- Rust 1.70+
- Chrome/Chromium browser (for full functionality)
- Docker & Docker Compose (for local Mistral service)
- Mistral API key (for cloud mode, optional with local mode)

## Architecture

```txt
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API      │    │   Browser       │    │   Mistral LLM   │
│   (Axum)        │◄──►│  (chromiumoxide)│    │   (Local/Cloud) │
│                 │    │                 │    │                 │
│ • Session Mgmt  │    │ • Page Control  │    │ • Task Planning │
│ • Task Planning │    │ • Element Inter │    │ • Action Break  │
│ • Data Extract  │    │ • Screenshot    │    │ • Fallback Plan │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                               │                 │
                                               ▼                 ▼
                                          ┌─────────┐   ┌─────────────┐
                                          │ Ollama  │   │ Mistral API │
                                          │ (Local) │   │  (Cloud)    │
                                          └─────────┘   └─────────────┘
```

### Local vs Cloud Mode

The service supports two modes for Mistral integration:

- **Local Mode**: Uses dockerized Ollama with local Mistral models
- **Cloud Mode**: Uses Mistral's cloud API (original behavior)

## API Endpoints

### Health Check

- `GET /health` - Server health status

### Browser Sessions

- `POST /browser/session` - Create a new browser session
- `GET /browser/session/{session_id}` - Get session status

### Browser Control

- `POST /browser/navigate` - Navigate to a URL
- `POST /browser/interact` - Perform browser actions
- `POST /browser/extract` - Extract data from page

### AI Automation

- `POST /automation/task` - Execute AI-planned automation tasks

## Browser Actions

The service supports various browser actions:

| Action | Description | Parameters |
|--------|-------------|------------|
| `Click` | Click an element | `selector` |
| `Type` | Type text into input | `selector`, `text` |
| `Wait` | Wait for duration | `duration_ms` |
| `WaitForElement` | Wait for element to appear | `selector`, `timeout_ms` |
| `Scroll` | Scroll page | `direction`, `pixels` |
| `Screenshot` | Capture screenshot | None |
| `GetPageSource` | Get HTML source | None |
| `ExecuteScript` | Run JavaScript | `script` |

## Configuration

### Environment Variables

#### General Configuration

- `RUST_LOG`: Logging level (debug, info, warn, error)
- `PORT`: Server port (default: 3000)

#### Mistral Configuration

- `MISTRAL_MODE`: Set to "local" for Ollama or "cloud" for API (default: cloud)

#### Local Mode (Ollama)

- `MISTRAL_LOCAL_ENDPOINT`: Ollama endpoint (default: http://localhost:11434)

#### Cloud Mode (Mistral API)

- `MISTRAL_API_KEY`: Your Mistral AI API key
- `MISTRAL_API_ENDPOINT`: Mistral API endpoint (default: https://api.mistral.ai/v1/chat/completions)

### Quick Setup Commands

#### Local Development with Dockerized Mistral

```bash
# Setup environment
make local-setup
# Edit .env file as needed
# Start all services
make docker-up
# Initialize models (first time only)
make init-models
```

#### Cloud Development with Mistral API

```bash
# Setup environment
make cloud-setup
# Edit .env file and add your API key
# Run the application
cargo run
```

### Docker Commands

```bash
# Build and start all services
make docker-up

# View logs
make docker-logs

# Stop services
make docker-down

# Check service status
make status

# Health checks
make health-local  # Check Ollama
make health-app    # Check application
```
