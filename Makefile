PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin

all: build

build:
	cargo build --release

install:
	install -Dm755 target/release/mntctl $(DESTDIR)$(BINDIR)/mntctl

clean:
	cargo clean

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/mntctl

.PHONY: all build clean install uninstall
