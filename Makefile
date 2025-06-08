.DEFAULT_GOAL := help

.PHONY: help
help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

# -- variables --------------------------------------------------------------------------------------

WARNINGS=RUSTDOCFLAGS="-D warnings"
BACKTRACE=RUST_BACKTRACE=1

# -- linting --------------------------------------------------------------------------------------

.PHONY: clippy
clippy: ## Runs Clippy with configs
	cargo clippy --workspace --all-targets $(ALL_FEATURES_BUT_ASYNC) -- -D warnings


.PHONY: format
format: ## Runs Format using nightly toolchain
	cargo +nightly fmt --all

# --- docs ----------------------------------------------------------------------------------------

.PHONY: doc
doc: ## Generates & checks documentation
	$(WARNINGS) cargo doc $(ALL_FEATURES_BUT_ASYNC) --keep-going --release

# --- testing -------------------------------------------------------------------------------------

.PHONY: test
test: ## Run all tests
	$(BACKTRACE) cargo nextest run --profile default


.PHONY: test-docs
test-docs: ## Run documentation tests
	$(WARNINGS) cargo test --doc


# --- checking ------------------------------------------------------------------------------------

.PHONY: check
check:
	cargo check --all-targets
