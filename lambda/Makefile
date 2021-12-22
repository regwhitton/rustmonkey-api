
#
# Default compile - if used by accident from this directory.
#
.PHONY: default
default:
	env PROFILE=debug CARGO_TARGET_DIR="$(PWD)/../target" $(MAKE) build

#
# Quick compile - used from Makefile above showing compiler output.
#
.PHONY: build
build:
	cargo build --target x86_64-unknown-linux-musl

#
# Run tests - used from Makefile above.
#
.PHONY: test
test:
	cargo test

#
# Not intended for use directly.  Used via "sam build" from Makefile above.
#
.PHONY: build-RustMonkeyFunction
build-RustMonkeyFunction:
	if [ "$(PROFILE)" = "release" ]; then \
	  cargo build --release --target x86_64-unknown-linux-musl; \
	else \
	  cargo build --target x86_64-unknown-linux-musl; \
	fi
	cp "$(CARGO_TARGET_DIR)/x86_64-unknown-linux-musl/$(PROFILE)/bootstrap" $(ARTIFACTS_DIR)
