# Makefile for LLM Web Agent

.PHONY: help build test clean docker-build docker-up docker-down docker-logs dev

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the Rust application
	cargo build --release

test: ## Run tests
	cargo test

clean: ## Clean build artifacts
	cargo clean

dev: ## Run the application in development mode
	RUST_LOG=debug cargo run

docker-build: ## Build Docker images
	docker-compose build

docker-up: ## Start all services with Docker Compose
	docker-compose up -d

docker-down: ## Stop all Docker services
	docker-compose down

docker-logs: ## View logs from all services
	docker-compose logs -f

docker-restart: ## Restart all Docker services
	docker-compose restart

docker-clean: ## Clean up Docker images and volumes
	docker-compose down -v
	docker system prune -f

local-setup: ## Setup for local development with Ollama
	@echo "Setting up local Mistral service..."
	@cp env.example .env
	@echo "Please edit .env file to configure your environment"
	@echo "Then run: make docker-up"

cloud-setup: ## Setup for cloud Mistral API
	@echo "Setting up cloud Mistral API..."
	@cp env.example .env
	@echo "Set MISTRAL_MODE=cloud in .env"
	@echo "Add your MISTRAL_API_KEY to .env"
	@echo "Then run: cargo run"

init-models: ## Initialize Mistral models in local Ollama (run after docker-up)
	@echo "Initializing Mistral models..."
	docker exec mistral-local ollama pull mistral:latest
	docker exec mistral-local ollama pull mistral:7b
	@echo "Models initialized successfully!"

status: ## Check service status
	docker-compose ps

# Health checks
health-local: ## Check local Ollama health
	curl -f http://localhost:11434/api/tags || echo "Ollama service not responding"

health-app: ## Check application health
	curl -f http://localhost:3000/health || echo "Application not responding" 