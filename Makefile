.PHONY: all
all: help

## Install

.PHONY: install
install: ## Installs git commit hooks
	@echo "Installing dependencies and setting up development environment..."
	@echo "Installing Rust dependencies..."
	@cargo build
	@echo "Setting up git hooks..."
	@./scripts/install-hooks.sh

.PHONY: udeps
udeps: ## Updates dependencies for the wasm target
	cargo udeps --target wasm32-unknown-unknown

## Code Quality
.PHONY: clippy
clippy: ## Runs cargo clippy
	cargo clippy --all-targets -- -D warnings

.PHONY: schema
schema: ## Runs json schema generation
	@echo "Generating schemas for all contracts..."
	@cd contracts && \
	for contract in */; do \
		if [ -d "$$contract" ]; then \
			echo "--- Generating schema for $$contract ---"; \
			cd "$$contract" && cargo schema && cd ..; \
		fi \
	done
	@echo "✅ Done"


## Build
.PHONY: build
build: ## Runs wasm build using cosmwasm docker optimizer
	docker run --rm -v "$(shell pwd)":/code \
	--mount type=volume,source="$(shell basename "$(shell pwd)")_cache",target=/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/optimizer:0.17.0

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

GREEN  := $(shell tput -Txterm setaf 2)
YELLOW := $(shell tput -Txterm setaf 3)
WHITE  := $(shell tput -Txterm setaf 7)
CYAN   := $(shell tput -Txterm setaf 6)
RESET  := $(shell tput -Txterm sgr0)

## Help:
.PHONY: help
help: ## Show this help
	@echo ''
	@echo 'Usage:'
	@echo '  ${YELLOW}make${RESET} ${GREEN}<target>${RESET}'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} { \
		if (/^[a-zA-Z_-]+:.*?##.*$$/) {printf "    ${YELLOW}%-20s${GREEN}%s${RESET}\n", $$1, $$2} \
		else if (/^## .*$$/) {printf "  ${CYAN}%s${RESET}\n", substr($$1,4)} \
		}' $(MAKEFILE_LIST)
