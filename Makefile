PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin

all: build

build:
	cargo build --release

install: build
	install -Dm755 target/release/mntctl $(DESTDIR)$(BINDIR)/mntctl

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/mntctl

.PHONY: all build install uninstall
