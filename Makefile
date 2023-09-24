CARGO ?= cargo

.PHONY: check
check:
	$(CARGO) check --workspace
