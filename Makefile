
.DEFAULT_GOAL:=help
SHELL:=/bin/bash
.PHONY: %
AWS_REGION:=eu-west-2

test: ## Build and run tests
	cd lambda && $(MAKE) test

build: ## Build lambda in release mode
	cd lambda && env PROFILE=release $(MAKE) build

build-debug: ## Build lambda in debug mode
	cd lambda && env PROFILE=debug $(MAKE) build

package: build ## Package stack for deployment
	env PROFILE=release CARGO_TARGET_DIR="$(PWD)/target" sam build

package-debug: build-debug ## Package stack for debugging locally
	env PROFILE=debug CARGO_TARGET_DIR="$(PWD)/target" sam build

deploy: package ## Deploy the stack onto AWS
	sam deploy --confirm-changeset

delete-stack: ## Remove the deployed stack from AWS
	sam delete

clean: ## Remove all the compiled binaries and packaging
	rm -rf target .aws-sam

local-deploy: package ## Start Localstack, deploy stack onto it, then wait
	env AWS_REGION=$(AWS_REGION) make-scripts/deploy-to-localstack.sh

local-test: package ## Start Localstack, deploy stack, run integration tests and stop
	env AWS_REGION=$(AWS_REGION) make-scripts/run-tests-on-localstack.sh

local-debug: package-debug ## Deploy debuggable stack on Localstack, wait for debugger to attach
	env AWS_REGION=$(AWS_REGION) make-scripts/debug-on-localstack.sh

validate: ## Validate the template.yaml file
	sam validate -t template.yaml


# tput colors
cyan := $(shell tput setaf 6)
reset := $(shell tput sgr0)
#
# Credits for Self documenting Makefile:
# https://www.thapaliya.com/en/writings/well-documented-makefiles/
# https://github.com/awinecki/magicfile/blob/main/Makefile
#
help: ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make $(cyan)[target ...]$(reset)\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(cyan)%-13s$(reset) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

