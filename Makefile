.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

# Native arch
BUILDARCH := $(shell uname -m)

.PHONY: start-node
start-node: ## Runs a network consisting of a single Aleph node.
	@echo "Starting aleph-network."
	@docker run --detach --rm --network host \
		--name aleph-network aleph-onenode-chain-${BUILDARCH}

.PHONY: stop-node
stop-node: ## Stops the local network.
	@echo "Stopping aleph-network."
	@docker stop aleph-network

.PHONY: restart-node
restart-node: stop-node start-node ## Restarts the local network.

.PHONY: build-node
# Build multi-CPU architecture images and publish them. rust alpine images support the linux/amd64 and linux/arm64/v8 architectures.
build-node: build-node-${BUILDARCH} ## Detects local arch and builds docker image
	@docker build --tag aleph-onenode-chain --file docker/Dockerfile docker

.PHONY: build-node-arm64
build-node-arm64:
	@docker buildx build --pull --platform linux/arm64/v8  -t aleph-onenode-chain-arm64 --load docker

.PHONY: build-node-x86_64
build-node-x86_64:
	@docker buildx build --pull --platform linux/amd64 -t aleph-onenode-chain-x86_64 --load docker

.PHONY: build-amm
build-amm: ## Builds AMM contracts.
	@cd amm && make build-all && cd ..

.PHONY: build-all
build-all: build-amm ## Builds all contracts.

.PHONY: check-amm
check-amm: ## Runs cargo (contract) check on AMM contracts.
	@cd amm && make check-amm && cd ..

.PHONY: check-all
check-all: build-and-wrap-all check-amm ## Runs cargo checks and unit tests on all contracts.
	@cargo test --quiet --locked --frozen --workspace

.PHONY: format
format: ## Formats contract files.
	@cargo fmt --all

CONTRACT_DATA = ./target/ink

.PHONY: wrap-all
wrap-all: ## Generates Rust wrappers for interacting with AMM contracts.
	@cd amm && make wrap-all && cd ..

.PHONY: build-and-wrap-all
build-and-wrap-all: build-all wrap-all ## Builds all contracts and generates code for contract interaction.

INK_DEV_IMAGE = public.ecr.aws/p6e8q1z1/ink-dev:2.1.0

.PHONY: check-all-dockerized
check-all-dockerized: ## Runs cargo checks and unit tests on all contracts in a container.
	@docker run --rm \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	$(INK_DEV_IMAGE) \
    	make check-all

.PHONY: build-and-wrap-all-dockerized
build-and-wrap-all-dockerized: ## Builds all contracts and generates code for contract interaction. Run in a container.
	@docker run --rm \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	$(INK_DEV_IMAGE) \
    	make build-and-wrap-all

.PHONY: build-dockerized
build-dockerized: ## Builds the contracts in a container.
	@docker run --rm \
		--name ink-dev \
		-v "$(shell pwd)":/code \
		$(INK_DEV_IMAGE) \
		make build-all

.PHONY: all-drink-dockerized
all-drink-dockerized: ## Runs the drink test in a container.
	@docker run --rm \
		--name ink-dev \
		-v "$(shell pwd)":/code \
		$(INK_DEV_IMAGE) \
		make all-drink

.PHONY: all-drink
all-drink: ## Runs the drink test.
	@cd amm && make all-drink && cd ..

.PHONY: all-dockerized
all-dockerized: ## Runs the drink test, unit tests and cargo checks on all contracts in a contrainer.
	@docker run --rm \
		--name ink-dev \
		-v "$(shell pwd)":/code \
		$(INK_DEV_IMAGE) \
		make all-drink-and-check

.PHONY: all-drink-and-check
all-drink-and-check: all-drink check-all ## Runs the drink test, unit tests and cargo checks on all contracts.
