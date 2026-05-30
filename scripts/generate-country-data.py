#!/usr/bin/env python3
"""Regenerate data/country_names.json from CLDR + system locale data.

Sources (in priority order, later overrides earlier):
  1. CLDR JSON release (unicode-org/cldr-json) territory names
  2. System LC_ADDRESS country_name extraction
  3. Manual overrides (for languages with no data from either source)

Usage:
  # From repo root:
  python3 scripts/generate-country-data.py
"""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import zipfile

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DATA_DIR = os.path.join(REPO_ROOT, "data")
OUTPUT = os.path.join(DATA_DIR, "country_names.json")

# GitHub API / download helpers
GH_API = "https://api.github.com/repos/unicode-org/cldr-json/releases/latest"
GH_DL = "https://github.com/unicode-org/cldr-json/releases/download"

# Manual override entries for languages with no CLDR or system data.
# These take highest priority.
MANUAL_OVERRIDES: dict[str, dict[str, str]] = {
    "crh": {"RU": "Русие Федерациясы", "UA": "Ukraina"},
    "kv": {"RU": "Россия"},
    "mhr": {"RU": "Россий"},
    "niu": {"NU": "Niuē", "NZ": "Niu Silani"},
    "quz": {"PE": "Piruw"},
    "shs": {"CA": "Kanata"},
}

# CLDR locale code to our @modifier suffix mapping.
# CLDR uses BCP47-style tags; we use lang@modifier convention.
SCRIPT_VARIANT_MAP: dict[str, str] = {
    "sr-Latn": "sr@latin",
    "sr-Cyrl": "sr",
    "uz-Cyrl": "uz@cyrillic",
    "uz-Latn": "uz",
    "az-Cyrl": "az@cyrillic",
    "az-Latn": "az",
    "bs-Cyrl": "bs@cyrillic",
    "bs-Latn": "bs",
    "ms-Arab": "ms@arabic",
    "ms-Latn": "ms",
    "kk-Arab": "kk@arabic",
    "kk-Cyrl": "kk",
    "ky-Arab": "ky@arabic",
    "ky-Cyrl": "ky",
    "tg-Cyrl": "tg",
    "tg-Persn": "tg@persian",
    "tt-Cyrl": "tt",
    "tt-Latn": "tt@latin",
    "sah-Cyrl": "sah",
    "sah-Latn": "sah@latin",
    "tk-Cyrl": "tk@cyrillic",
    "tk-Latn": "tk",
    "ug-Arab": "ug",
    "ug-Cyrl": "ug@cyrillic",
    "ug-Latn": "ug@latin",
}

# Known main locales whose territories file we should check.
# The CLDR release has dozens of locales; we filter to those that look
# meaningful. This list is built dynamically from the extracted zip.
SKIP_LOCALES = {"root", "und", "zxx"}


def log(msg: str) -> None:
    print(f"  [{os.path.basename(sys.argv[0])}] {msg}", flush=True)


def fetch_latest_cldr_version() -> str:
    """Determine latest CLDR release tag from GitHub API."""
    log("Fetching latest CLDR version from GitHub API...")
    try:
        req = urllib.request.Request(GH_API, headers={"Accept": "application/json", "User-Agent": "xfce-aero-lang-changer"})
        with urllib.request.urlopen(req, timeout=30) as resp:
            data = json.loads(resp.read().decode())
            tag = data["tag_name"]
            log(f"Latest CLDR version: {tag}")
            return tag
    except Exception as e:
        log(f"Warning: could not fetch latest version ({e}), falling back to 48.2.0")
        return "48.2.0"


def download_cldr_zip(version: str, dest: str) -> str:
    """Download and extract CLDR JSON full zip. Returns the path to extracted dir."""
    zip_name = f"cldr-{version}-json-full.zip"
    url = f"{GH_DL}/{version}/{zip_name}"
    zip_path = os.path.join(dest, zip_name)
    extract_dir = os.path.join(dest, f"cldr-{version}")

    if os.path.exists(extract_dir):
        log(f"Already extracted: {extract_dir}")
        return extract_dir

    log(f"Downloading {url}...")
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "xfce-aero-lang-changer"})
        with urllib.request.urlopen(req, timeout=120) as resp:
            with open(zip_path, "wb") as f:
                f.write(resp.read())
    except Exception as e:
        log(f"Error downloading: {e}")
        sys.exit(1)

    log(f"Extracting {zip_path}...")
    with zipfile.ZipFile(zip_path, "r") as zf:
        zf.extractall(extract_dir)

    # Find cldr-localenames-full first (has the territory data we need)
    entries = os.listdir(extract_dir)
    if "cldr-localenames-full" in entries:
        return os.path.join(extract_dir, "cldr-localenames-full")
    # Fallback: any cldr-* dir that has main/
    for entry in entries:
        if entry.startswith("cldr-") and os.path.isdir(os.path.join(extract_dir, entry, "main")):
            return os.path.join(extract_dir, entry)

    return extract_dir


def parse_cldr_territories(base_dir: str) -> dict[str, dict[str, str]]:
    """Parse CLDR territory data from extracted JSON files.

    base_dir should be the extracted cldr-localenames-full directory
    (or a parent containing a 'main/' subdir).

    Returns a dict mapping our lang codes (e.g. "sr", "sr@latin") to
    {territory_code: localized_name}.
    """
    result: dict[str, dict[str, str]] = {}
    main_dir = os.path.join(base_dir, "main")
    if not os.path.isdir(main_dir):
        for root, dirs, files in os.walk(base_dir):
            if "territories.json" in files and root.endswith("/main"):
                main_dir = root
                break
        else:
            log(f"Error: could not find main/ directory with territories.json in {base_dir}")
            sys.exit(1)

    for locale_id in sorted(os.listdir(main_dir)):
        locale_dir = os.path.join(main_dir, locale_id)
        if not os.path.isdir(locale_dir):
            continue
        if locale_id in SKIP_LOCALES:
            continue

        territories_file = os.path.join(locale_dir, "territories.json")
        if not os.path.isfile(territories_file):
            continue

        try:
            with open(territories_file, encoding="utf-8") as f:
                data = json.load(f)
        except (json.JSONDecodeError, UnicodeDecodeError):
            continue

        # Navigate: main -> {localeId} -> localeDisplayNames -> territories
        main_block = data.get("main", {})
        # The locale key in CLDR may differ from the directory name
        # (e.g. dir "af" has key "af" inside, but some variant locales
        # might have normalized keys). Try the exact locale_id first,
        # then fall back to the first key in main_block.
        loc_block = main_block.get(locale_id)
        if loc_block is None:
            # Fallback: use the first (and typically only) key
            for k in main_block:
                loc_block = main_block[k]
                break
            if loc_block is None:
                continue
        ldn = loc_block.get("localeDisplayNames", {})
        territories = ldn.get("territories", {})
        if not territories:
            continue

        # Map CLDR locale code to our @modifier convention
        lang_code = map_locale_code(locale_id)
        if lang_code is None:
            continue

        # Only keep alpha territory codes (e.g. US, GB, RU) and numeric UN M.49
        # regions (e.g. 001, 002, 150). The numeric ones are for region groupings.
        filtered: dict[str, str] = {}
        for code, name in territories.items():
            if isinstance(name, str) and (code.isalpha() or code.isdigit()):
                filtered[code] = name

        if filtered:
            if lang_code in result:
                result[lang_code].update(filtered)
            else:
                result[lang_code] = filtered

    return result


def map_locale_code(locale_id: str) -> str | None:
    """Map a CLDR BCP47 locale ID to our lang@modifier convention."""
    # Check explicit script variants first
    if locale_id in SCRIPT_VARIANT_MAP:
        return SCRIPT_VARIANT_MAP[locale_id]

    # Handle ca-ES-VALENCIA -> ca@valencia
    if re.match(r"^[a-z]{2,3}-[A-Z]{2}-VALENCIA$", locale_id):
        base = locale_id.split("-")[0]
        return f"{base}@valencia"

    # Generic pattern: lang[-Script][-Region]
    m = re.match(r"^([a-z]{2,3})(?:-[A-Z][a-z]{3})?(?:-[A-Z]{2})?$", locale_id)
    if m:
        return m.group(1)

    # If locale has a script variant we don't know about, add it as @script
    m = re.match(r"^([a-z]{2,3})-([A-Z][a-z]{3})(?:-[A-Z]{2})?$", locale_id)
    if m:
        base = m.group(1)
        script = m.group(2).lower()
        return f"{base}@{script}"

    return None


def extract_system_locales() -> dict[str, dict[str, str]]:
    """Extract country names from system LC_ADDRESS.

    Runs `locale -ck LC_ADDRESS country_name` for each installed locale
    that has a country_name entry.

    Returns dict mapping lang codes to {territory_code: localized_name}.
    """
    result: dict[str, dict[str, str]] = {}

    log("Extracting system locale country names...")
    try:
        output = subprocess.check_output(
            ["locale", "-a"],
            timeout=30,
            text=True,
            stderr=subprocess.DEVNULL,
        )
        locales = [l.strip() for l in output.splitlines() if l.strip()]
    except (subprocess.SubprocessError, FileNotFoundError):
        log("Warning: locale command not available, skipping system data")
        return result

    for loc in locales:
        if loc in SKIP_LOCALES or loc == "C" or loc == "POSIX":
            continue

        # Map locale code to our format
        # Normalize: e.g. sr_RS@latin -> sr_RS@latin, sr_RS.utf8 -> sr_RS
        # We use the locale -ck output directly for country_name
        try:
            cout_bytes = subprocess.check_output(
                ["locale", "-ck", "LC_ADDRESS", "country_name"],
                timeout=10,
                stderr=subprocess.DEVNULL,
                env={**os.environ, "LC_ALL": loc},
            )
            # locale output may be in the locale's encoding; try UTF-8 first
            try:
                cout = cout_bytes.decode("utf-8")
            except UnicodeDecodeError:
                cout = cout_bytes.decode("latin-1", errors="replace")
        except subprocess.SubprocessError:
            continue

        name: str | None = None
        for line in cout.splitlines():
            # Format: "country_name="Россия""
            if line.startswith("country_name="):
                val = line.split("=", 1)[1].strip()
                if val:
                    name = val.strip('"')
                break

        if not name:
            continue

        # Determine territory code from locale itself: sr_RS -> RS
        parts = loc.split("_")
        if len(parts) >= 2:
            territory = parts[1].split(".")[0].split("@")[0].upper()
            if len(territory) == 2 and territory.isalpha():
                # Determine our lang code
                lang = parts[0].lower()
                # Extract modifiers from locale
                modifier = None
                if "@" in loc:
                    modifier = loc.split("@")[1].lower()

                lang_code = lang
                if modifier in ("latin", "latn", "iqtelif"):
                    lang_code = f"{lang}@latin"
                elif modifier in ("cyrillic", "cyrl"):
                    lang_code = f"{lang}@cyrillic"
                elif modifier:
                    lang_code = f"{lang}@{modifier}"

                if lang_code not in result:
                    result[lang_code] = {}
                result[lang_code][territory] = name

    return result


def main() -> None:
    os.makedirs(DATA_DIR, exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="cldr-") as tmpdir:
        # 1. Fetch CLDR data
        version = fetch_latest_cldr_version()
        cldr_dir = download_cldr_zip(version, tmpdir)

        # 2. Parse CLDR territory data
        log("Parsing CLDR territory data...")
        data = parse_cldr_territories(cldr_dir)
        log(f"  CLDR data: {len(data)} languages")

        # 3. Extract system locale data
        log("Extracting system locale data...")
        sys_data = extract_system_locales()
        log(f"  System data: {len(sys_data)} languages")

        # 4. Merge: CLDR first, then system data overrides
        for lang, territories in sys_data.items():
            if lang in data:
                data[lang].update(territories)
                log(f"  System override applied to: {lang}")
            else:
                data[lang] = territories
                log(f"  System data (new lang): {lang}")

        # 5. Apply manual overrides (highest priority)
        for lang, territories in MANUAL_OVERRIDES.items():
            if lang in data:
                data[lang].update(territories)
            else:
                data[lang] = territories
            log(f"  Manual override applied to: {lang}")

        # 6. Write output, sorted for reproducibility
        log(f"Writing {OUTPUT} ({len(data)} languages)...")
        sorted_data = dict(sorted(data.items()))
        for lang in sorted_data:
            sorted_data[lang] = dict(sorted(sorted_data[lang].items()))

        with open(OUTPUT, "w", encoding="utf-8") as f:
            json.dump(sorted_data, f, ensure_ascii=False, indent=1)
            f.write("\n")

    log("Done!")


if __name__ == "__main__":
    main()
