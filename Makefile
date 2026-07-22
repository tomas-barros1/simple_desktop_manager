PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
APPDIR ?= $(PREFIX)/share/applications
NAME = simple_menu_manager
DESKTOP_FILE = dev.simplemenu.DesktopManager.desktop

.PHONY: all build release check test clean install uninstall

all: build

build:
	cargo build

release:
	cargo build --release

check:
	cargo check

test:
	cargo test

clean:
	cargo clean

install: release
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 target/release/$(NAME) $(DESTDIR)$(BINDIR)/$(NAME)
	install -d $(DESTDIR)$(APPDIR)
	install -m 644 $(DESKTOP_FILE) $(DESTDIR)$(APPDIR)/$(DESKTOP_FILE)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(NAME)
	rm -f $(DESTDIR)$(APPDIR)/$(DESKTOP_FILE)
