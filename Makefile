SHELL := /bin/bash
.DEFAULT_GOAL := help

ROOT := $(CURDIR)
COMPOSE_FILE := $(ROOT)/infra/compose/docker-compose.yml
COMPOSE_ENV := $(if $(wildcard $(ROOT)/.env),--env-file $(ROOT)/.env,)
COMPOSE := docker compose $(COMPOSE_ENV) -f $(COMPOSE_FILE)
COMPOSE_DEV := docker compose $(COMPOSE_ENV) -f $(COMPOSE_FILE) -f $(ROOT)/infra/compose/docker-compose.dev.yml
COMPOSE_OBS := docker compose $(COMPOSE_ENV) -f $(COMPOSE_FILE) -f $(ROOT)/infra/compose/docker-compose.observability.yml
COMPOSE_TLS := docker compose $(COMPOSE_ENV) -f $(COMPOSE_FILE) -f $(ROOT)/infra/compose/docker-compose.tls.yml
COMPOSE_EGRESS := docker compose $(COMPOSE_ENV) -f $(COMPOSE_FILE) -f $(ROOT)/infra/compose/docker-compose.egress.yml
WEB := $(ROOT)/scripts/run-web.sh

.PHONY: help setup hooks compose-validate up up-dev up-obs up-tls up-egress down reset logs ps migrate seed seed-demo \
	fmt fmt-check lint typecheck test check frontend-build frontend-test frontend-lint \
	e2e bdd bdd-list bdd-set-api-key load doctor generate-key backup restore-drill openapi asyncapi sbom sow-lint \
	docs-check arch-fitness crate-fitness compose-validate compose-isolation db-check supply-chain-test clean

help: ## List targets with one-line descriptions
	@grep -E '^[a-zA-Z0-9_-]+:.*?## ' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-22s %s\n", $$1, $$2}'

setup: ## Install dependencies and tool images (Docker only)
	scripts/run-rust.sh cargo fetch
	$(WEB) npm ci

hooks: ## Install commit hooks (lefthook or native git hooks)
	scripts/hooks/install.sh

compose-validate: ## Validate the Compose base and every profile
	$(COMPOSE) --profile core config -q
	$(COMPOSE_DEV) --profile core config -q
	$(COMPOSE_OBS) --profile core --profile observability config -q
	$(COMPOSE_TLS) --profile core --profile tls config -q
	$(COMPOSE_EGRESS) --profile core --profile egress config -q
	@echo "compose-validate: all profiles valid"

up: ## Build and start the core stack (WP-03)
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is missing" >&2; exit 1; }
	$(COMPOSE) --profile core up -d --build

up-dev: ## Start the core stack with development ports and mounts
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is missing" >&2; exit 1; }
	$(COMPOSE_DEV) --profile core up -d --build

up-obs: ## Start the core and observability profiles (WP-03/WP-20)
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is missing" >&2; exit 1; }
	$(COMPOSE_OBS) --profile core --profile observability up -d --build

up-tls: ## Start the core stack behind Caddy TLS
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is missing" >&2; exit 1; }
	$(COMPOSE_TLS) --profile core --profile tls up -d --build

up-egress: ## Start the core stack with the sandbox egress proxy
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is missing" >&2; exit 1; }
	$(COMPOSE_EGRESS) --profile core --profile egress up -d --build

down: ## Stop containers, preserving volumes
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is owned by WP-03 and does not exist yet" >&2; exit 1; }
	$(COMPOSE) down

reset: ## Stop, remove volumes, re-up, migrate and seed demo
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is owned by WP-03 and does not exist yet" >&2; exit 1; }
	$(COMPOSE) down -v
	$(MAKE) up
	$(MAKE) migrate
	$(MAKE) seed-demo

logs: ## Follow core service logs
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is owned by WP-03 and does not exist yet" >&2; exit 1; }
	$(COMPOSE) logs -f --tail=200

ps: ## Show service status and health
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is owned by WP-03 and does not exist yet" >&2; exit 1; }
	$(COMPOSE) ps

migrate: ## Apply sqlx migrations (WP-03/WP-04)
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is owned by WP-03 and does not exist yet" >&2; exit 1; }
	$(COMPOSE) run --rm migrate

seed: ## Seed reference data (WP-04)
	@test -f $(COMPOSE_FILE) || { echo "error: $(COMPOSE_FILE) is owned by WP-03 and does not exist yet" >&2; exit 1; }
	$(COMPOSE) run --rm seed

seed-demo: ## Seed the deterministic demo dataset (WP-04)
	scripts/seed-demo.sh

fmt: ## Format Rust and frontend code in Docker
	scripts/run-rust.sh cargo fmt --all
	$(WEB) npm run format

fmt-check: ## Check formatting in Docker
	scripts/run-rust.sh cargo fmt --all --check
	$(WEB) npm run format:check

lint: ## Clippy with -D warnings and ESLint in Docker
	scripts/run-rust.sh cargo clippy --workspace --all-targets -- -D warnings
	$(WEB) npm run lint

test: ## Rust and frontend tests in Docker
	scripts/run-rust.sh cargo test --workspace
	$(WEB) npm test

typecheck: ## Type-check the frontend without emitting
	$(WEB) npx tsc -p tsconfig.app.json --noEmit

check: fmt-check lint typecheck test sow-lint docs-check openapi-drift arch-fitness crate-fitness compose-validate compose-isolation ## Fast local gate
	@echo "check: OK"

frontend-build: ## Build the Angular app and enforce budgets
	$(WEB) npm run build

frontend-test: ## Run frontend unit tests
	$(WEB) npm test

frontend-lint: ## Run ESLint and Prettier checks
	$(WEB) npm run lint
	$(WEB) npm run format:check

e2e: ## Run Playwright end-to-end tests in the Playwright container
	@test -f $(ROOT)/web/playwright.config.ts || { echo "error: Playwright is owned by WP-17/WP-21 and is not configured yet" >&2; exit 1; }
	.tools/run-e2e.sh

bdd: ## Run the BDD E2E suite (all, SUITE=id, or UP_TO=id)
	@AILAB_BDD_VIA_MAKE=1 AILAB_BDD_SUITE="$(SUITE)" AILAB_BDD_UP_TO="$(UP_TO)" scripts/run-bdd.sh

bdd-list: ## List the six BDD suites and their scenario counts
	@scripts/bdd-suites.sh --list

bdd-set-api-key: ## Load the DeepSeek API key for live BDD runs (hidden prompt)
	@scripts/bdd-set-api-key.sh

load: ## Run k6 load scenarios in the k6 container
	.tools/run-load.sh

doctor: ## Validate Docker, ports, disk, env, images and connectivity
	scripts/doctor.sh
generate-key: ## Generate the local master key and runner token in .env (Docker)
	@scripts/generate-key.sh

backup: ## Dump the database and write an artifact manifest (WP-03/WP-20)
	scripts/backup.sh

restore-drill: ## Restore into a scratch database and verify (WP-03/WP-20)
	@test -n "$(BACKUP)" || { echo "usage: make restore-drill BACKUP=artifacts/backups/<file>.dump" >&2; exit 1; }
	scripts/restore-drill.sh "$(BACKUP)"

openapi: ## Regenerate OpenAPI and client types (WP-16)
	scripts/gen-openapi.sh

openapi-drift: ## Fail when an OpenAPI path is not served by the router
	scripts/run-python.sh .tools/check-openapi-drift.py --root .

asyncapi: ## Regenerate AsyncAPI (WP-16)
	scripts/gen-asyncapi.sh

sbom: ## Generate SBOMs for built images in the Syft container (WP-22)
	@test -n "$(IMAGES)" || { echo "usage: make sbom IMAGES='ailab/api:1.0.0 ...'" >&2; exit 1; }
	@mkdir -p artifacts/sbom
	@for image in $(IMAGES); do \
		docker run --rm -v /var/run/docker.sock:/var/run/docker.sock -v "$(ROOT)/artifacts/sbom:/out" anchore/syft:latest "$$image" -o cyclonedx-json="/out/$$(echo $$image | tr '/:' '__').cdx.json"; \
	done
	@echo "sbom: written to artifacts/sbom"

sow-lint: ## Validate SOW statuses, links, checklists and dependencies
	scripts/sow-lint.sh

docs-check: ## Validate the harness and relative documentation links
	scripts/harness-lint.sh
	.tools/check-doc-links.sh

arch-fitness: ## Validate context ownership, dependencies and table access
	.tools/check-architecture-fitness.sh
	.tools/architecture-fitness-self-test.sh

crate-fitness: ## Validate crate direction, acyclicity, unsafe and unwrap gates
	.tools/check-crate-dependencies.sh

compose-isolation: ## Assert Compose isolation, hardening and resource limits
	.tools/test-compose-isolation.sh

db-check: ## Run migration, constraint and analytics database suites
	.tools/test-migrations.sh
	.tools/test-constraints.sh
	.tools/test-analytics-views.sh

supply-chain-test: ## Negative tests for npm audit and cargo-deny
	.tools/test-supply-chain.sh

clean: ## Remove build artifacts, never source
	@docker volume rm ailab-cargo-target >/dev/null 2>&1 || true
	rm -rf target web/dist web/.angular web/coverage artifacts/backups artifacts/sbom
	@echo "clean: build artifacts removed"
