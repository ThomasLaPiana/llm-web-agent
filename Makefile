# Makefile for LLM Web Agent

.PHONY: help build test clean docker-build docker-up docker-down docker-logs dev

help: ## Show this help message
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

###################################
## Tests, Linting and Formatting ##
###################################
test:
	cargo test

lint:
	cargo clippy

format:
	cargo fmt

###############
## App Setup ##
###############
build: ## Build the Rust application
	cargo build --release

clean: ## Clean build artifacts
	cargo clean

dev: ## Run the application in development mode
	RUST_LOG=debug cargo run

docker-build: ## Build Docker images
	docker-compose build

docker-up: ## Start all services with Docker Compose
	docker-compose up --wait

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

###################
## Health checks ##
###################
health-local: ## Check local Ollama health
	curl -f http://localhost:11434/api/tags || echo "Ollama service not responding"

health-app: ## Check application health
	curl -f http://localhost:3000/health || echo "Application not responding"

#############################
## Load testing with Drill ##
#############################
load-test-check: ## Check if server is running before load tests
	@echo "Checking if LLM Web Agent server is running..."
	@curl -f -s http://localhost:3000/health > /dev/null && echo "✅ Server is running and healthy!" || (echo "❌ Server is not running! Start with: make dev" && exit 1)

load-test-light: load-test-check ## Run light load test (50 iterations, 5 concurrent users)
	@echo "🚀 Running light load test..."
	drill --stats --benchmark drill-light.yml
	@echo "✅ Light load test completed! Check drill-light-report.html for results."

load-test-standard: load-test-check ## Run standard load test (100 iterations, 10 concurrent users)
	@echo "🚀 Running standard load test..."
	drill --benchmark drill-config.yml
	@echo "✅ Standard load test completed! Check drill-report.html for results."

load-test-heavy: load-test-check ## Run heavy stress test (500 iterations, 50 concurrent users)
	@echo "⚠️  Running heavy load test (500 iterations, 50 concurrent users)..."
	@echo "⚠️  This test may put significant load on your system!"
	@read -p "Are you sure you want to continue? (y/N): " confirm && [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ] || (echo "Heavy load test cancelled." && exit 1)
	drill --benchmark drill-heavy.yml
	@echo "✅ Heavy load test completed! Check drill-heavy-report.html for results."

load-test-clean: ## Clean up load test report files
	@echo "🧹 Cleaning up load test reports..."
	rm -f drill-*-report.html drill-*-report.json
	@echo "✅ Load test reports cleaned up."

load-test-all: load-test-light load-test-standard ## Run all standard load tests (light + standard)
	@echo "✅ All load tests completed!" 