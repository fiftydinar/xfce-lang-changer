# xfce-aero-lang-changer

A GUI language switcher for XFCE with Aero-style theming, built with [fltk-rs](https://github.com/fltk-rs/fltk-rs).

Lists all available system locales with human-readable language and region
names, and integrates into **XFCE Settings** as a standalone panel entry.

## Features

- Browse and search all installed locales (`locale -a`)
- Human-friendly display: *"English (United States)  —  en_US.UTF-8"*
- Real-time filter as you type
- Aero-style gradient header theme
- Writes locale to files under `$XDG_CONFIG_HOME` (or `~/.config`):
  - `locale.conf`
  - `environment.d/99-lang.conf`
  - `xfce4/environment`
- Sets `LANG` via `systemctl --user set-environment`
- Prompts to log out after applying — locale is set at process startup and cannot be changed at runtime
- Dynamic linking against system `fltk` (static build also available)

## Requirements

- XFCE desktop environment
- [fltk](https://archlinux.org/packages/extra/x86_64/fltk/) system package (for dynamic linking)
- System locales generated (see `locale -a` / `locale.gen`)

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
