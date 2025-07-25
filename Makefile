SHELL:=/bin/bash
MAKEFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
PROJECT_ROOT := $(dir $(MAKEFILE_PATH))

.PHONY: default
default: help

.PHONY: help
help: ## Display this help screen
	@grep -h -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

.PHONY: fmt
fmt: ## Format the project
	@cargo fmt --all

.PHONY: lint
lint: ## Lint the project
	@cargo clippy --all --all-targets
