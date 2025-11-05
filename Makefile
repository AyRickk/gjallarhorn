.PHONY: help build run test dev-up dev-down logs clean docker-build deploy-k8s

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build the Rust application
	cargo build --release

run: ## Run the application locally
	cargo run

test: ## Run tests
	cargo test

dev-up: ## Start development environment with Docker Compose
	docker-compose up -d
	@echo ""
	@echo "Services started:"
	@echo "  - API:             http://localhost:8080"
	@echo "  - Grafana:         http://localhost:3000 (admin/admin)"
	@echo "  - Prometheus:      http://localhost:9090"
	@echo "  - Keycloak:        http://localhost:8180 (admin/admin)"
	@echo "  - Webhook Monitor: http://localhost:8081"
	@echo "  - Feedback UI:     http://localhost:8082"
	@echo ""

dev-down: ## Stop development environment
	docker-compose down

dev-restart: ## Restart development environment
	docker-compose restart

logs: ## Show logs from all services
	docker-compose logs -f

logs-api: ## Show API logs only
	docker-compose logs -f feedback-api

clean: ## Clean build artifacts
	cargo clean
	docker-compose down -v

docker-build: ## Build Docker image
	docker build -t feedback-api:latest .

docker-push: ## Push Docker image (configure registry first)
	docker tag feedback-api:latest your-registry/feedback-api:latest
	docker push your-registry/feedback-api:latest

deploy-k8s: ## Deploy to Kubernetes
	kubectl apply -k k8s/

deploy-k8s-delete: ## Delete Kubernetes deployment
	kubectl delete -k k8s/

k8s-logs: ## Show Kubernetes logs
	kubectl logs -n feedback-system -l app=feedback-api -f

k8s-status: ## Show Kubernetes deployment status
	kubectl get all -n feedback-system

db-migrate: ## Run database migrations
	sqlx migrate run

db-reset: ## Reset database (WARNING: deletes all data)
	docker-compose exec postgres psql -U feedback_user -d feedback_db -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
	sqlx migrate run

fmt: ## Format code
	cargo fmt

lint: ## Run linter
	cargo clippy -- -D warnings

check: fmt lint test ## Run all checks (format, lint, test)
