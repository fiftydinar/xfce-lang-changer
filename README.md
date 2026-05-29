# xfce-aero-lang-changer

A GUI language switcher for XFCE with Aero-style theming, built with [fltk-rs](https://github.com/fltk-rs/fltk-rs).

Lists all available system locales with human-readable language and region
names, and integrates into **XFCE Settings** as a standalone panel entry.

Used for [xfce-aerolike](https://github.com/fiftydinar/xfce-aerolike/tree/main/recipes) project, but it can probably work for other distros too.

## Features

- Browse and search all installed locales (`locale -a`)
- Two-column table: language name (in its own script) and locale code right-aligned
- Shows native language names (autonyms) — e.g. "Deutsch" instead of "German", "français" instead of "French"
- Locale variant awareness: prefers Latin names when `@latin` is in the locale code
- Real-time filter as you type (searches both name and code fields)
- Aero-style gradient header with bold title and light subtitle
- Custom-drawn table with alternating row colors, subtle grid lines, and soft border
- Thin rounded "Current:" label with Aero teal accent
- Writes `LANG` to `$XDG_CONFIG_HOME/locale.conf` (or `~/.config/locale.conf`) — read by `pam_systemd` at login
- Prompts to log out after applying — locale is set at process startup and cannot be changed at runtime
- Dynamic linking against system `fltk` (static build also available)

## Requirements

- XFCE desktop environment
- System locales generated (see `locale -a` / `locale.gen`)
- [fltk](https://archlinux.org/packages/extra/x86_64/fltk/) system package for dynamic linking
- [Noto Sans](https://archlinux.org/packages/extra/any/noto-sans/) font recommended for proper RTL and wide script support (falls back to default FLTK font if missing)

## Build & Install

Build with dynamic linking (default):

```sh
make
sudo make install
```

Build with static linking (bundled fltk):

```sh
make LINK=static
sudo make install
```

## Uninstall

```sh
sudo make uninstall
```

## License

Apache 2.0

Part of the [xfce-aerolike](https://github.com/fiftydinar/xfce-aerolike) project.
