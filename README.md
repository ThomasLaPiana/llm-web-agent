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
- Mistral API key (optional, fallback mode available)

## Architecture

```txt
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   REST API      │    │   Browser       │    │   Mistral LLM   │
│   (Axum)        │◄──►│  (chromiumoxide)│    │   (MCP)         │
│                 │    │                 │    │                 │
│ • Session Mgmt  │    │ • Page Control  │    │ • Task Planning │
│ • Task Planning │    │ • Element Inter │    │ • Action Break  │
│ • Data Extract  │    │ • Screenshot    │    │ • Fallback Plan │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

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

- `MISTRAL_API_KEY`: Your Mistral AI API key
- `MISTRAL_API_ENDPOINT`: Mistral API endpoint (default: https://api.mistral.ai/v1/chat/completions)
- `RUST_LOG`: Logging level (debug, info, warn, error)
- `PORT`: Server port (default: 3000)
