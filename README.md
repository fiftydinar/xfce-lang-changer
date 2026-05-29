# xfce-aero-lang-changer

A GUI language switcher for XFCE with Aero-style theming, built with [fltk-rs](https://github.com/fltk-rs/fltk-rs).

Part of the [xfce-aerolike](https://github.com/fiftydinar/xfce-aerolike) project.

Lists all available system locales with native language names and region codes in a two-column table. Real-time search filters both columns. Applies the selected locale to `$XDG_CONFIG_HOME/locale.conf` and prompts to log out (locale takes effect on next session).

## Build & Install

```sh
make
sudo make install
```

Static build: `make LINK=static`

## Requirements

- XFCE
- System locales generated (`locale -a`)
- [fltk](https://archlinux.org/packages/extra/x86_64/fltk/) (for dynamic linking)
- [Noto Sans](https://archlinux.org/packages/extra/any/noto-sans/) (for RTL script support)

## Uninstall

```sh
sudo make uninstall
```
