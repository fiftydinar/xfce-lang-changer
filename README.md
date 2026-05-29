# xfce-lang-changer

A GUI language switcher for XFCE with Aero-style theming, built with [fltk-rs](https://github.com/fltk-rs/fltk-rs).

Lists all available system locales with human-readable language and region
names, and integrates into **XFCE Settings** as a standalone panel entry.

## Features

- Browse and search all installed locales (`locale -a`)
- Human-friendly display: *"English (United States)  —  en_US.UTF-8"*
- Real-time filter as you type
- Aero-style gradient header theme
- Writes locale to:
  - `~/.config/locale.conf`
  - `~/.config/environment.d/99-lang.conf`
  - `~/.config/xfce4/environment`
- Sets `LANG` via `systemctl --user set-environment`
- Prompts to log out after applying — locale is set at process startup and cannot be changed at runtime

## Requirements

- XFCE desktop environment
- System locales generated (see `locale -a` / `locale.gen`)

## Build & Install

```sh
cargo build --release
sudo make install
```

Or without `make`:

```sh
cargo build --release
sudo install -Dm755 target/release/xfce-lang-changer /usr/local/bin/xfce-lang-changer
sudo install -Dm644 xfce-lang-changer.desktop /usr/local/share/applications/xfce-lang-changer.desktop
```

## Uninstall

```sh
sudo make uninstall
```

## License

Apache 2.0

Part of the [xfce-aerolike](https://github.com/fiftydinar/xfce-aerolike) project.
