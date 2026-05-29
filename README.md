# xfce-lang-changer

A GUI language changer for XFCE with Aero-style theming, built with [fltk-rs](https://github.com/fltk-rs/fltk-rs).

Integrated into XFCE Settings as a standalone panel entry.

## Features

- Lists all available (generated) locales from `locale -a`
- Human-readable locale names with language and country
- Aero-themed GUI matching the xfce-aerolike design
- Per-user locale change (no root required for switching)
- Real-time environment update via `systemctl --user set-environment`
- One-click logout for full application of changes

## How it works

1. Scans `locale -a` for installed locales
2. User selects a locale from the list
3. Writes `LANG=xx_XX.UTF-8` to:
   - `~/.config/locale.conf` (PAM environment)
   - `~/.config/environment.d/99-lang.conf` (systemd user)
   - `~/.config/xfce4/environment` (XFCE session)
4. Calls `systemctl --user set-environment LANG=...` (immediate for new processes)
5. Prompts to log out (full session restart needed)

## Building from source

```bash
cd xfce-lang-changer
cargo build --release
```

## Installation

```bash
chmod +x install.sh
./install.sh
```

## Integration with xfce-aerolike

### 1. Recipe modification

In `recipes/recipe.yml`, replace the locale section:

**Before:**
```yaml
# Set default language to English and generate locale
- |
  sed -i 's/^#\s*\(en_US.UTF-8 UTF-8\)/\1/' /etc/locale.gen
- "locale-gen"
- "echo 'LANG=en_US.UTF-8' > /etc/locale.conf"
```

**After:**
```yaml
# Generate common locales for the language changer
- |
  for loc in \
    en_US.UTF-8 UTF-8 \
    de_DE.UTF-8 UTF-8 \
    fr_FR.UTF-8 UTF-8 \
    es_ES.UTF-8 UTF-8 \
    it_IT.UTF-8 UTF-8 \
    pt_BR.UTF-8 UTF-8 \
    pt_PT.UTF-8 UTF-8 \
    nl_NL.UTF-8 UTF-8 \
    sv_SE.UTF-8 UTF-8 \
    da_DK.UTF-8 UTF-8 \
    nb_NO.UTF-8 UTF-8 \
    fi_FI.UTF-8 UTF-8 \
    pl_PL.UTF-8 UTF-8 \
    cs_CZ.UTF-8 UTF-8 \
    sk_SK.UTF-8 UTF-8 \
    hu_HU.UTF-8 UTF-8 \
    ro_RO.UTF-8 UTF-8 \
    bg_BG.UTF-8 UTF-8 \
    ru_RU.UTF-8 UTF-8 \
    uk_UA.UTF-8 UTF-8 \
    el_GR.UTF-8 UTF-8 \
    hr_HR.UTF-8 UTF-8 \
    sr_RS.UTF-8 UTF-8 \
    sl_SI.UTF-8 UTF-8 \
    ja_JP.UTF-8 UTF-8 \
    ko_KR.UTF-8 UTF-8 \
    zh_CN.UTF-8 UTF-8 \
    zh_TW.UTF-8 UTF-8 \
    th_TH.UTF-8 UTF-8 \
    vi_VN.UTF-8 UTF-8 \
    id_ID.UTF-8 UTF-8 \
    ar_SA.UTF-8 UTF-8 \
    he_IL.UTF-8 UTF-8 \
    fa_IR.UTF-8 UTF-8 \
    tr_TR.UTF-8 UTF-8 \
    hi_IN.UTF-8 UTF-8 \
    bn_BD.UTF-8 UTF-8 \
    ta_IN.UTF-8 UTF-8 \
    te_IN.UTF-8 UTF-8 \
    mr_IN.UTF-8 UTF-8; \
  do \
    sed -i "s/^# *\($loc\)/\1/" /etc/locale.gen; \
  done
- "locale-gen"
- "echo 'LANG=en_US.UTF-8' > /etc/locale.conf"
```

### 2. Add the xfce-lang-changer binary

Build the Rust project and include the binary in the image.

### 3. Add the desktop file

Copy `xfce-lang-changer.desktop` to `files/usr/share/applications/xfce-lang-changer.desktop`
