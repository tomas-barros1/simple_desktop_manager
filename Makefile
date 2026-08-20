PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
APPDIR ?= $(PREFIX)/share/applications
NAME = simple_menu_manager
DESKTOP_FILE = dev.simplemenu.DesktopManager.desktop
TARGET_BIN = target/release/$(NAME)

.PHONY: all build release check test clean install uninstall

all: release

build:
	cargo build

release: $(TARGET_BIN)

$(TARGET_BIN):
	cargo build --release

check:
	cargo check

test:
	cargo test

clean:
	cargo clean

install:
	@if [ ! -f $(TARGET_BIN) ]; then \
		echo "Binary not found, building release..."; \
		cargo build --release; \
	fi
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 $(TARGET_BIN) $(DESTDIR)$(BINDIR)/$(NAME)
	install -d $(DESTDIR)$(APPDIR)
	install -m 644 $(DESKTOP_FILE) $(DESTDIR)$(APPDIR)/$(DESKTOP_FILE)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(NAME)
	rm -f $(DESTDIR)$(APPDIR)/$(DESKTOP_FILE)
