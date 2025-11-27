.PHONY: build frontend backend clean dev test docker-build docker-run docker-stop docker-logs docker-clean install format check generate-types

# Build the entire project
build: frontend backend

# Build frontend (generates types first)
frontend:
	cd frontend && npm run build

# Build backend with optimizations
backend:
	RUSTFLAGS="-C target-cpu=native" cargo build --release --jobs $(shell nproc)

# Development mode (run both frontend and backend)
dev:
	@echo "Starting development servers..."
	@echo "Frontend: http://localhost:5173"
	@echo "Backend will be available after build"
	@make frontend &
	@cargo run &
	@wait

# Clean build artifacts
clean:
	cd frontend && rm -rf dist node_modules/.vite ../static
	cargo clean
	rm -rf target

# Run tests
test:
	cd frontend && npm test
	cargo test

# Docker commands
docker-build:
	@echo "Building Docker image with BuildKit optimizations..."
	DOCKER_BUILDKIT=1 docker build --build-arg BUILDKIT_INLINE_CACHE=1 -t clewdr-kill:latest .

docker-run:
	@echo "Starting ClewdR Kill Edition container..."
	docker-compose up -d

docker-stop:
	@echo "Stopping ClewdR Kill Edition container..."
	docker-compose down

docker-logs:
	docker-compose logs -f clewdr-kill

docker-clean:
	@echo "Cleaning Docker resources..."
	docker-compose down -v
	docker image prune -f
	docker volume prune -f

docker-rebuild: docker-stop docker-clean docker-build docker-run

# Production deployment
deploy: docker-build
	@echo "Deploying to production..."
	docker-compose up -d --force-recreate

# Generate types only
generate-types:
	cd scripts && node generate_types.ts

# Install dependencies
install:
	cd frontend && npm install
	cargo build

# Format code
format:
	cd frontend && npm run lint -- --fix
	cargo fmt

# Check for issues
check:
	cd frontend && npm run lint
	cargo check
	cargo clippy -- -D warnings

# Performance testing
perf-test:
	@echo "Running performance tests..."
	cd tests/perf && k6 run k6-smoke.js

# Backup data
backup:
	@echo "Creating backup..."
	@mkdir -p backups
	@docker run --rm -v clewdr_data:/data -v $(PWD)/backups:/backup alpine \
		tar czf /backup/clewdr-data-$(shell date +%Y%m%d-%H%M%S).tar.gz -C /data .
	@echo "Backup created in backups/ directory"

# Restore data
restore:
	@echo "Available backups:"
	@ls -la backups/
	@echo "To restore, run: make restore-file BACKUP=filename.tar.gz"

restore-file:
	@if [ -z "$(BACKUP)" ]; then echo "Usage: make restore-file BACKUP=filename.tar.gz"; exit 1; fi
	@docker run --rm -v clewdr_data:/data -v $(PWD)/backups:/backup alpine \
		tar xzf /backup/$(BACKUP) -C /data
	@echo "Restored from $(BACKUP)"

# Health check
health:
	@curl -f http://localhost:8484/api/health || echo "Service is not healthy"

# Show container stats
stats:
	docker stats clewdr-kill --no-stream

# Quick setup for new environment
setup: install build docker-build
	@echo "Setup complete! Run 'make docker-run' to start the service."
