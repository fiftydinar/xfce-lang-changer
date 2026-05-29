PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
APPDIR ?= $(DATADIR)/applications

TARGET ?= release
BINARY := target/$(TARGET)/xfce-lang-changer

.PHONY: all build install uninstall clean

all: build

build:
	cargo build --release

$(BINARY): build

install: $(BINARY)
	install -Dm755 $(BINARY) $(DESTDIR)$(BINDIR)/xfce-lang-changer
	install -Dm644 xfce-lang-changer.desktop $(DESTDIR)$(APPDIR)/xfce-lang-changer.desktop

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/xfce-lang-changer
	rm -f $(DESTDIR)$(APPDIR)/xfce-lang-changer.desktop

clean:
	cargo clean
