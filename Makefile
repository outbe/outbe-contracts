.PHONY: install
install:
	@echo "Installing dependencies and setting up development environment..."
	@echo "Installing Rust dependencies..."
	@cargo build
	@echo "Setting up git hooks..."
	@./scripts/install-hooks.sh

.PHONY: clippy
clippy:
	cargo clippy --all-targets -- -D warnings

.PHONY: udeps
udeps:
	cargo +nightly udeps --target wasm32-unknown-unknown

.PHONY: schema
schema:
	@echo "Generating schemas for all contracts..."
	@for contract_package in $$(cargo metadata --no-deps --quiet | jq -r '.packages[] | select(.targets[].kind[] == "example") | .name'); do \
		echo "--- Generating schema for '$$contract_package' ---"; \
		cargo schema -p $$contract_package; \
	done

.PHONY: build
build:
	docker run --rm -v "$(shell pwd)":/code \
	--mount type=volume,source="$(shell basename "$(shell pwd)")_cache",target=/target \
	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
	cosmwasm/optimizer:0.17.0
