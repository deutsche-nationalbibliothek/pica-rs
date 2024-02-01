.DEFAULT_GOAL := build

# Don't use implicit rules or variables.
MAKEFLAGS += -rR

DESTDIR=
CARGO ?= cargo
PREFIX=/usr/local
BINDIR=$(PREFIX)/bin

check:
	$(CARGO) check --workspace --all-features

build:
	$(CARGO) build --workspace --all-features

clippy:
	$(CARGO) clippy --workspace --all-features

test:
	$(CARGO) test --workspace --all-features --no-fail-fast

careful:
	$(CARGO) +nightly careful test --workspace --all-features

fmt:
	$(CARGO) fmt --all -- --check

doc:
	$(CARGO) doc --no-deps

release:
	$(CARGO) build --workspace --all-features --release
	$(CARGO) test --workspace --all-features --release

install: release
	sudo install -Dm755 target/release/pica $(DESTDIR)$(BINDIR)

clean:
	$(CARGO) clean

.PHONY: build clean check clippy test fmt doc release install
