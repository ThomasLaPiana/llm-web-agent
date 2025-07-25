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
- Chrome/Chromium browser
- Mistral API key (optional, fallback mode available)

### Setup and Run
```bash
# Clone and setup
git clone <repository-url>
cd llm-web-agent

# Setup development environment (creates .env, builds project)
make setup

# Start the server
make run

# In another terminal, run tests
make test
```

### Available Commands (Makefile)

```bash
make help                # Show all available commands

# Testing
make test               # Run unit + integration tests (recommended)
make test-unit          # Fast unit tests (type safety, serialization)
make test-integration   # API tests (requires server or skips gracefully)
make test-browser       # Full browser automation tests (requires Chrome)
make test-all           # Run all tests including browser tests
make test-ci            # CI/CD test suite

# Development
make run                # Start the server
make dev                # Start server with debug logging
make build              # Build in debug mode
make build-release      # Build in release mode
make check              # Fast code check
make fmt                # Format code
make lint               # Run linter
make docs               # Generate documentation

# Utility
make example            # Show API usage examples
make setup              # Setup development environment
make clean              # Clean build artifacts
```

## Architecture

```
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

## Testing

The project includes comprehensive unit and integration tests.

### Quick Testing
```bash
make test              # Run recommended test suite
make test-unit         # Fast unit tests only
make test-integration  # API integration tests
```

### Detailed Testing Options
```bash
# Unit Tests - Always fast, no dependencies
make test-unit

# Integration Tests - Requires server or skips gracefully
make test-integration

# Browser Tests - Requires Chrome/Chromium
make test-browser

# All Tests - Complete test suite
make test-all

# CI/CD Tests - Suitable for automated environments
make test-ci
```

### Test Architecture
- **Unit Tests** (`tests/unit_tests.rs`): Fast tests for core logic and type safety
- **Integration Tests** (`tests/integration_tests.rs`): API endpoint testing with graceful fallbacks
- **Browser Tests**: Full end-to-end testing (marked with `#[ignore]`)

### Running in CI/CD
```bash
# Recommended for CI/CD - fast and reliable
make test-ci

# Or use cargo directly
cargo test --test unit_tests
cargo test --test integration_tests
```

## Manual Installation & Setup

If you prefer not to use the Makefile:

### 1. Clone and Build
```bash
git clone <repository-url>
cd llm-web-agent
cargo build --release
```

### 2. Environment Configuration
```bash
cp .env.example .env
# Edit .env with your Mistral API key
```

### 3. Run the Server
```bash
cargo run
# Server will start on http://localhost:3000
```

### 4. Manual Testing
```bash
# Unit tests
cargo test --test unit_tests

# Integration tests
cargo test --test integration_tests

# Browser tests
cargo test --test integration_tests -- --ignored
```

## Usage Examples

### 1. Create Browser Session
```bash
curl -X POST http://localhost:3000/browser/session
# Response: {"session_id": "uuid-here"}
```

### 2. Navigate to Website
```bash
curl -X POST http://localhost:3000/browser/navigate \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "your-session-id",
    "url": "https://example.com"
  }'
```

### 3. Perform Browser Actions
```bash
# Click an element
curl -X POST http://localhost:3000/browser/interact \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "your-session-id",
    "action": {
      "type": "Click",
      "params": {"selector": "#submit-button"}
    }
  }'

# Type text into input
curl -X POST http://localhost:3000/browser/interact \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "your-session-id",
    "action": {
      "type": "Type",
      "params": {"selector": "#email", "text": "user@example.com"}
    }
  }'
```

### 4. AI-Powered Automation
```bash
curl -X POST http://localhost:3000/automation/task \
  -H "Content-Type: application/json" \
  -d '{
    "session_id": "your-session-id",
    "task_description": "Fill out the contact form with my email",
    "target_url": "https://example.com/contact",
    "context": {"email": "user@example.com"}
  }'
```

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

### Browser Configuration

The browser launches with optimized settings for automation:
- Sandbox disabled for Docker compatibility
- GPU acceleration disabled
- Shared memory usage disabled
- Custom Chrome arguments for stability

## Error Handling

The API returns structured error responses:

```json
{
  "error": "Error description",
  "status": 400
}
```

Common error types:
- `BrowserError`: Browser automation issues
- `SessionNotFound`: Invalid session ID
- `MCPError`: LLM communication problems
- `SerializationError`: JSON parsing issues
- `InternalError`: Server-side errors

## Development

### Project Structure
```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library interface and server logic
├── browser.rs        # Browser automation logic
├── mcp.rs           # Mistral LLM integration
└── types.rs         # Request/response types

tests/
├── unit_tests.rs     # Unit tests for types and serialization
└── integration_tests.rs  # API integration tests

Makefile              # Development commands and testing
```

### Development Workflow

```bash
# Setup environment
make setup

# Development with hot reload (requires cargo-watch)
make watch

# Run tests during development
make test-unit        # Fast feedback
make test            # Full validation

# Code quality
make check           # Fast compilation check
make lint            # Linting
make fmt            # Code formatting
```

### Adding New Actions

1. Add action variant to `BrowserAction` enum in `types.rs`
2. Implement action logic in `browser.rs`
3. Add tests in `tests/unit_tests.rs`
4. Run `make test` to validate
5. Update API documentation

### Testing Strategy

- **Unit Tests**: Fast tests for core logic and type safety
- **Integration Tests**: API endpoint testing with graceful fallbacks
- **Browser Tests**: Full end-to-end testing (marked with `#[ignore]`)

Use `make test` for daily development, `make test-ci` for automated environments.

## Docker Support

```dockerfile
# Dockerfile included for containerized deployment
FROM rust:1.70 as builder
# ... build steps

FROM chromium-browser:latest
# ... runtime setup
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run the test suite: `make test`
6. Run validation: `make validate`
7. Submit a pull request

## Troubleshooting

### Browser Launch Issues
- Ensure Chrome/Chromium is installed
- Check browser arguments in `browser.rs`
- Verify permissions for sandbox settings

### API Connection Issues
- Verify Mistral API key is correct
- Check network connectivity
- Review API endpoint configuration

### Test Issues
- Unit tests should always pass: `make test-unit`
- Integration tests require server to be running: `make test-integration`
- Browser tests need Chrome/Chromium installed: `make test-browser`

### Development Issues
- Use `make check` for fast compilation checking
- Use `make lint` to catch common issues
- Use `make fmt` to fix formatting

### Performance Optimization
- Use headless mode for production
- Implement session pooling for high load
- Consider browser instance reuse
