WASM_TARGET := wasm32v1-none
CONTRACTS := $(patsubst contracts/%,%,$(wildcard contracts/*))

.PHONY: all
all: test build

.PHONY: build
build: ## Build all contracts to WASM
	cargo build --target $(WASM_TARGET) --release

.PHONY: build-optimized
build-optimized: ## Build all contracts to WASM with the Stellar CLI (smaller, optimized)
	stellar contract build --workspace

.PHONY: test
test: ## Run the full test suite
	cargo test --workspace

.PHONY: lint
lint: ## Run clippy across the workspace
	cargo clippy --workspace --all-targets -- -D warnings

.PHONY: fmt
fmt: ## Format the workspace
	cargo fmt --all

.PHONY: clean
clean:
	cargo clean

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2}'