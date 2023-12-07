CARGO ?= cargo

.PHONY: check
check:
	$(CARGO) check --workspace --all-features

.PHONY: clippy
clippy:
	$(CARGO) clippy --workspace --all-features -- \
		-D warnings -D rust-2021-compatibility -D future-incompatible \
		-W unreachable-pub

.PHONY: test
test:
	$(CARGO) test --workspace --all-features --no-fail-fast \
		-- --test-threads=1
