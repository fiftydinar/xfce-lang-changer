# xfce-aero-lang-changer

A GUI language switcher for XFCE with Aero-style theming, built with [fltk-rs](https://github.com/fltk-rs/fltk-rs).

Part of the [xfce-aerolike](https://github.com/fiftydinar/xfce-aerolike) project.

Lists all available system locales with native language names and region codes in a two-column table. Real-time search filters both columns with Latin ↔ Cyrillic transliteration (for Serbian/Croatian/Bosnian/Montenegrin). Applies the selected locale to `$XDG_CONFIG_HOME/locale.conf` and prompts to log out (locale takes effect on next session).

## Compatibility

Locale switching writes to `~/.config/locale.conf` (XDG_CONFIG_HOME). This file is sourced by `/etc/profile.d/locale.sh` on **Arch Linux** and derivatives (Manjaro, EndeavourOS), taking effect on next login.

| Distro | Support | Notes |
|---|---|---|
| Arch Linux | ✅ Full | via profile.d/locale.sh |
| Fedora / RHEL | ❌ No | GNOME uses GSettings; locale.conf ignored |
| Debian / Ubuntu | ❌ No | uses /etc/default/locale |
| openSUSE | ❌ No | uses /etc/sysconfig/language |

| Desktop Env | Behavior | Notes |
|---|---|---|
| XFCE | ✅ Works | reads LANG from session environment |
| GNOME | ❌ Bypassed | uses own GSettings, ignores locale.conf |
| KDE Plasma | ❌ Bypassed | uses ~/.config/plasma-localerc |

This app is primarily aimed at **Arch Linux with XFCE**.

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
