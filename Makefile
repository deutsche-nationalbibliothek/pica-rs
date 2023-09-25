CARGO ?= cargo

.PHONY: check
check:
	$(CARGO) check --workspace --all-features

.PHONY: test
test:
	$(CARGO) test --workspace --all-features --no-fail-fast
