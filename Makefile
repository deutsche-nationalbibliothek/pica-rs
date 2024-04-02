.DEFAULT_GOAL := build

# Don't use implicit rules or variables.
MAKEFLAGS += -rR

DESTDIR=
CARGO ?= cargo
HUGO ?= hugo
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

check-fmt:
	$(CARGO) fmt --all -- --check

doc:
	$(CARGO) doc --no-deps

docs:
	mkdir -p target/docs
	$(CARGO) doc --all --no-deps --workspace --target-dir target/docs/api
	$(HUGO) --minify --gc --source docs --destination ../target/docs/book
	echo '<meta http-equiv="refresh" content="0; url=doc/pica/index.html"><a href=doc/pica/index.html">Redirect</a>' \
		> target/docs/api/index.html
	echo '<meta http-equiv="refresh" content="0; url=book/index.html"><a href=book/index.html">Redirect</a>' \
		> target/docs/index.html

release:
	$(CARGO) build --workspace --all-features --release
	$(CARGO) test --workspace --all-features --release

install: release
	sudo install -Dm755 target/release/pica $(DESTDIR)$(BINDIR)

clean:
	$(CARGO) clean

.PHONY: build clean check clippy test fmt doc docs release install
