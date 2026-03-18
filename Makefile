PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin

all: build

build:
	cargo build --release

install:
	install -Dm755 target/release/mntctl $(DESTDIR)$(BINDIR)/mntctl

test:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test

clean:
	cargo clean

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/mntctl

.PHONY: all build test clean install uninstall
