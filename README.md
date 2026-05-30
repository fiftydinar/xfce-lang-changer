# xfce-aero-lang-changer

<table>
  <tr>
    <td><img width="100%" alt="Snimak ekrana od 2026-05-29 22-39-56" src="https://github.com/user-attachments/assets/b236b344-4a2d-4dcb-b936-276cc77fec5a" /></td>
    <td><img width="100%" alt="Snimak ekrana od 2026-05-29 22-41-54" src="https://github.com/user-attachments/assets/4fc43bdc-7de2-49cd-852f-7fe310b49ce5" /></td>
  </tr>
</table>

A language picker for Arch + XFCE X11 desktops, built with FLTK in Aero style.

Presents all generated system languages in a table with native names. Filter as you type with built-in Latin ↔ Cyrillic transliteration. Selecting a language writes it to `locale.conf` and offers to log out so the change takes effect on next login.

Made primarely for the purposes of my XFCE custom image distribution:  
https://github.com/fiftydinar/xfce-aerolike

```sh
make && sudo make install
```

## How it works

The tool writes `LANG=<language>` to `$XDG_CONFIG_HOME/locale.conf`. On Arch Linux and derivatives, this file is sourced by `/etc/profile.d/locale.sh` at login. It also injects the same `LANG` export into `~/.xprofile` for sessions that read that file.

The selected language is applied on the **next login**, not immediately. After picking a language, the dialog lets you either log out now or do it later.

## Compatibility guard

Before opening the picker, the app checks for known incompatibilities — Wayland sessions, non-Arch distros, GNOME/KDE/Deepin desktop environments, and similar scenarios where `locale.conf` may be ignored or overridden. If any are detected, the app displays the details and exits.

This guard exists because the tool writes to a file that only certain session stacks respect. Running it blindly on an incompatible setup gives the illusion of a working language change while actually doing nothing.

## Requirements

- Linux with XFCE (or another desktop that reads `locale.conf`/`.xprofile`)
- System languages generated (check with `locale -a`)
- FLTK library for dynamic builds (`fltk-git` on Arch AUR repository)
- Noto Sans font for RTL script rendering
- Rust toolchain to build from source

## Building

| Mode | Command |
|------|---------|
| Dynamic (system FLTK) | `make` |
| Static (bundled FLTK) | `make LINK=static` |

## Uninstalling

```sh
sudo make uninstall
```
