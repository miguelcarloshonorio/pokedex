
.PHONY: help

help: ## Show help
	@fgrep -h "##" $(MAKEFILE_LIST) | sort | fgrep -v fgrep | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

test: ## Run project unit tests
	@ cargo test

run: ## Run project
	@ cargo run

build: ## Build project
	@ cargo build