PREFIX ?= /usr/local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
APPDIR ?= $(DATADIR)/applications

TARGET ?= release
BINARY := target/$(TARGET)/xfce-aero-lang-changer

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
	install -Dm755 $(BINARY) $(DESTDIR)$(BINDIR)/xfce-aero-lang-changer
	mkdir -p $(DESTDIR)$(APPDIR)
	while IFS= read -r line || [ -n "$$line" ]; do \
		case "$$line" in \
			Exec=*) echo "Exec=$(BINDIR)/xfce-aero-lang-changer" ;; \
			*) echo "$$line" ;; \
		esac; \
	done < xfce-aero-lang-changer.desktop > $(DESTDIR)$(APPDIR)/xfce-aero-lang-changer.desktop

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/xfce-aero-lang-changer
	rm -f $(DESTDIR)$(APPDIR)/xfce-aero-lang-changer.desktop

clean:
	cargo clean
