PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
APPDIR ?= $(DATADIR)/applications

TARGET ?= release
BINARY := target/$(TARGET)/xfce-lang-changer

# Linking mode: static (bundled fltk) or dynamic (system fltk)
LINK ?= dynamic
CARGO_ARGS := --release
CARGO_ARGS += $(if $(filter dynamic,$(LINK)),--no-default-features,--features bundled)

.PHONY: all build install uninstall clean

all: build

build:
	cargo build $(CARGO_ARGS)

$(BINARY): build

install: $(BINARY)
	install -Dm755 $(BINARY) $(DESTDIR)$(BINDIR)/xfce-lang-changer
	install -Dm644 xfce-lang-changer.desktop $(DESTDIR)$(APPDIR)/xfce-lang-changer.desktop

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/xfce-lang-changer
	rm -f $(DESTDIR)$(APPDIR)/xfce-lang-changer.desktop

clean:
	cargo clean
