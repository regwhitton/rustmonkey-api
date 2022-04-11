
.DEFAULT_GOAL:=help
SHELL:=/bin/bash
.PHONY: %

build: ## Build lambda dependencies and executable
	@if [ "$(PROFILE)" = "release" ]; then \
	  cargo build --release --target x86_64-unknown-linux-musl; \
	else \
	  cargo build --target x86_64-unknown-linux-musl; \
	fi

test: ## Build and run tests
	env RUST_BACKTRACE=1 cargo test

# 
# Used by "sam build" from Makefile in directory above.
# Do not use directly.
#
# Relies on the binary already having been built (doing it here
# hides the compiler output).  This just copies the result into place.
#
# During "sam build" this whole directory is copied elsewhere,
# so the CARGO_TARGET_DIR environment variable is set by other Makefile
# so that binaries can be found.
#
build-RustMonkeyFunction:
	@if [ "$(PROFILE)" = "release" ]; then \
	  cp "$(CARGO_TARGET_DIR)/x86_64-unknown-linux-musl/$(PROFILE)/bootstrap" \
	      $(ARTIFACTS_DIR); \
	else \
	  cp "$(CARGO_TARGET_DIR)/x86_64-unknown-linux-musl/debug/bootstrap" \
	      $(ARTIFACTS_DIR)/lambda_debug.exe; \
	  echo '#!/bin/bash' > $(ARTIFACTS_DIR)/bootstrap; \
	  echo 'lldb-gdbserver "*:7737" -- ./lambda_debug.exe' \
	      >> $(ARTIFACTS_DIR)/bootstrap; \
	  chmod +x $(ARTIFACTS_DIR)/bootstrap; \
	fi

# tput colors
cyan := $(shell tput setaf 6)
reset := $(shell tput sgr0)

help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make $(cyan)[target ...]$(reset)\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  $(cyan)%-15s$(reset) %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

