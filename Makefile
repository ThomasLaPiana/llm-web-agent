# LLM Web Agent Makefile
# Provides convenient commands for building, testing, and running the web automation service

.PHONY: help build run clean test test-unit test-integration test-browser test-all check fmt lint install dev

# Default target
help: ## Show this help message
	@echo "LLM Web Agent - Available Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""

# Build Commands
build: ## Build the project in debug mode
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

# Run Commands
run: ## Start the web automation server
	@echo "Starting LLM Web Agent server on http://localhost:3000"
	cargo run

dev: ## Run server with debug logging
	@echo "Starting LLM Web Agent server with debug logging"
	RUST_LOG=debug cargo run

# Test Commands
test-unit: ## Run fast unit tests (type safety, serialization)
	@echo "Running unit tests..."
	cargo test --test unit_tests

test-integration: ## Run integration tests (API endpoints, requires server or skips)
	@echo "Running integration tests..."
	@echo "Note: If server is not running, tests will skip gracefully"
	cargo test --test integration_tests

test-browser: ## Run browser automation tests (requires Chrome/Chromium)
	@echo "Running browser integration tests..."
	@echo "Note: These tests require Chrome/Chromium to be installed"
	cargo test --test integration_tests -- --ignored

test: ## Run all tests including browser tests
	@echo "Running all tests (unit + integration + browser)..."
	make test-unit
	make test-integration
	make test-browser

# Development Commands
lint: ## Run clippy linter
	cargo clippy -- -D warnings

fix: ## Auto-fix linting issues where possible
	cargo clippy --fix --allow-dirty --allow-staged

# Documentation Commands
docs: ## Generate and open documentation
	cargo doc --open

# Server Management
start-bg: ## Start server in background for testing
	@echo "Starting server in background..."
	@cargo run > /dev/null 2>&1 &
	@echo "Server started. Use 'make stop' to stop it."

stop: ## Stop background server
	@echo "Stopping server..."
	@pkill -f llm-web-agent || echo "No server process found"

# Environment Setup
setup: ## Setup development environment
	@echo "Setting up development environment..."
	@if [ ! -f .env ]; then cp .env.example .env && echo "Created .env from .env.example"; fi
	@cargo build
	@echo "Setup complete! Run 'make run' to start the server."

# Watch Commands (requires cargo-watch: cargo install cargo-watch)
watch: ## Watch for changes and run tests automatically
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x "test --test unit_tests"; \
	else \
		echo "cargo-watch not installed. Run: cargo install cargo-watch"; \
	fi

watch-integration: ## Watch for changes and run integration tests
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x "test --test integration_tests"; \
	else \
		echo "cargo-watch not installed. Run: cargo install cargo-watch"; \
	fi 