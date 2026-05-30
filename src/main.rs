use std::{cell::RefCell, collections::HashMap, path::PathBuf, process::Command, rc::Rc};
use fltk::{
    app, button::Button, dialog,
    enums::{Align, CallbackTrigger, Color, FrameType},
    frame::Frame, input::Input, prelude::*,
    table::{TableContext, TableRow}, text::{TextBuffer, TextDisplay, WrapMode},
    window::Window,
};
use fltk_theme::{widget_themes, WidgetTheme, ThemeType};

const AERO_BORDER: Color = Color::from_hex(0x09554E);

#[derive(Clone)]
struct Trans {
    window_title: String,
    header_title: String,
    header_subtitle: String,
    search_hint: String,
    current_fmt: String,
    apply: String,
    quit: String,
    lang_changed_title: String,
    lang_changed_msg_fmt: String,
    later: String,
    logout_btn: String,
    ok: String,
    compat_title: String,
    no_locales: String,
    compat_quit: String,
    error_fmt: String,
    warning_wayland: String,
    warning_gnome_fmt: String,
    warning_kde: String,
    warning_deepin: String,
    warning_non_arch: String,
}

const LANG_JSON: &str = include_str!("lang.json");

fn trans_from_map(m: &HashMap<String, String>) -> Trans {
    fn g(m: &HashMap<String, String>, key: &str) -> String {
        m.get(key).cloned().unwrap_or_default()
    }
    Trans {
        window_title: g(m, "window_title"),
        header_title: g(m, "header_title"),
        header_subtitle: g(m, "header_subtitle"),
        search_hint: g(m, "search_hint"),
        current_fmt: g(m, "current_fmt"),
        apply: g(m, "apply"),
        quit: g(m, "quit"),
        lang_changed_title: g(m, "lang_changed_title"),
        lang_changed_msg_fmt: g(m, "lang_changed_msg_fmt"),
        later: g(m, "later"),
        logout_btn: g(m, "logout_btn"),
        ok: g(m, "ok"),
        compat_title: g(m, "compat_title"),
        no_locales: g(m, "no_locales"),
        compat_quit: g(m, "compat_quit"),
        error_fmt: g(m, "error_fmt"),
        warning_wayland: g(m, "warning_wayland"),
        warning_gnome_fmt: g(m, "warning_gnome_fmt"),
        warning_kde: g(m, "warning_kde"),
        warning_deepin: g(m, "warning_deepin"),
        warning_non_arch: g(m, "warning_non_arch"),
    }
}

fn load_translations() -> Trans {
    let all: HashMap<String, HashMap<String, String>> =
        serde_json::from_str(LANG_JSON).expect("Embedded lang.json is invalid");

    let lang_raw = std::env::var("LANG").unwrap_or_default();

    // Extract @modifier from anywhere — handles both sr_RS.UTF-8@latin and sr_RS@latin.UTF-8
    let (modifier, base_with_encoding) = match lang_raw.find('@') {
        Some(pos) => {
            let after = &lang_raw[pos + 1..];
            let mod_part = after.split('.').next().unwrap_or(after);
            (Some(mod_part.to_lowercase()), &lang_raw[..pos])
        }
        None => (None, lang_raw.as_str()),
    };

    let normalized = base_with_encoding.split('.').next().unwrap_or(base_with_encoding).to_lowercase();
    let mut candidates = vec![normalized.clone()];

    if let Some(pos) = normalized.find(&['_', '-'][..]) {
        let lang = &normalized[..pos];
        let rest = &normalized[pos + 1..];
        if !rest.is_empty() {
            candidates.push(format!("{}-{}", lang, rest));
        }
        if let Some(v) = &modifier {
            candidates.push(format!("{}@{}", lang, v));
            candidates.push(format!("{}-{}", lang, v));
            // Map common @modifier words to ISO 15924 script codes
            if v == "latin" || v == "latn" {
                candidates.push(format!("{}-Latn", lang));
            }
        }
        candidates.push(lang.to_string());
    } else if let Some(v) = &modifier {
        candidates.push(format!("{}@{}", normalized, v));
        candidates.push(format!("{}-{}", normalized, v));
        if v == "latin" || v == "latn" {
            candidates.push(format!("{}-Latn", normalized));
        }
    }

    for c in &candidates {
        if let Some(m) = all.get(c) {
            return trans_from_map(m);
        }
    }

    trans_from_map(all.get("en").expect("Missing English translations"))
}

fn tr_fmt(template: &str, pairs: &[(&str, &str)]) -> String {
    let mut s = template.to_string();
    for (key, val) in pairs {
        let placeholder = format!("{{{}}}", key);
        s = s.replace(&placeholder, val);
    }
    s
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join(".config")
    })
}

fn get_current_locale() -> String {
    if let Ok(lang) = std::env::var("LANG") {
        return lang;
    }
    let conf_path = config_dir().join("locale.conf");
    if let Ok(content) = std::fs::read_to_string(conf_path) {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("LANG=") {
                return val.trim().to_string();
            }
        }
    }
    "en_US.UTF-8".to_string()
}

fn get_available_locales() -> Vec<(String, String)> {
    let mut raw: Vec<String> = Vec::new();
    if let Ok(output) = Command::new("locale").arg("-a").output() {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let l = line.trim();
                if !l.is_empty() && l != "C" && l != "POSIX" && l != "C.utf8"
                    && (l.contains(".utf8") || l.contains(".UTF-8"))
                {
                    raw.push(l.to_string());
                }
            }
        }
    }

    let mut groups: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for loc in &raw {
        let without_encoding = loc.split('.').next().unwrap_or(loc);
        let modifier = loc.split('@').nth(1).unwrap_or("");
        let group_key = if modifier.is_empty() {
            without_encoding.to_string()
        } else {
            format!("{}@{}", without_encoding, modifier)
        };
        let is_utf8 = loc.contains(".utf8") || loc.contains(".UTF-8");
        let entry = groups.entry(group_key);
        use std::collections::btree_map::Entry;
        match entry {
            Entry::Occupied(mut e) => {
                if is_utf8 {
                    e.insert(loc.clone());
                }
            }
            Entry::Vacant(e) => {
                e.insert(loc.clone());
            }
        }
    }

    groups.into_values().map(|loc| {
        let human = locale_to_human_name(&loc);
        (human, loc)
    }).collect()
}

fn lang_name(code: &str, modifier: Option<&str>) -> String {
    if let Some(lang) = isolang::Language::from_639_1(code) {
        let autonym = lang.to_autonym()
            .map(|s| s.to_string())
            .unwrap_or_else(|| lang.to_name().to_string());
        match modifier {
            Some("latin") | Some("Latn") if is_cyrillic(autonym.chars().next().unwrap_or(' ')) => {
                transliterate(&autonym)
            }
            Some("cyrillic") | Some("Cyrl") if !is_cyrillic(autonym.chars().next().unwrap_or(' ')) => {
                transliterate(&autonym)
            }
            _ => autonym,
        }
    } else {
        code.to_string()
    }
}

fn country_name(code: &str, lang: &str) -> String {
    let code = code.split('@').next().unwrap_or(code);
    use i18n_country_translations::*;
    let _ = register_locale(lang);
    get_name_for_locale(lang, code)
        .unwrap_or_else(|| rust_iso3166::from_alpha2(code).map(|c| c.name).unwrap_or(code).to_string())
}

fn locale_to_human_name(locale: &str) -> String {
    let modifier = locale.split('@').nth(1);
    let without_modifier = locale.split('@').next().unwrap_or(locale);
    let lang_part = without_modifier.split('.').next().unwrap_or(without_modifier);
    let parts: Vec<&str> = lang_part.split('_').collect();
    let lang_code = parts[0];
    let region_code = parts.get(1).copied().unwrap_or("");

    let lang_name = lang_name(lang_code, modifier);
    let region_name = if !region_code.is_empty() {
        let mut c = country_name(region_code, lang_code);
        let first_char = c.chars().next().unwrap_or(' ');
        match modifier {
            Some("latin") | Some("Latn") if is_cyrillic(first_char) => {
                c = transliterate(&c);
            }
            Some("cyrillic") | Some("Cyrl") if !is_cyrillic(first_char) => {
                c = transliterate(&c);
            }
            _ => {}
        }
        format!(" ({})", c)
    } else {
        String::new()
    };
    format!("{}{}", lang_name, region_name)
}

fn is_cyrillic(c: char) -> bool {
    matches!(c, '\u{0400}'..='\u{04FF}' | '\u{0500}'..='\u{052F}')
}

fn transliterate(s: &str) -> String {
    use cyrla::ConverterBuilder;
    use std::sync::OnceLock;
    static CONVERTER: OnceLock<cyrla::Converter> = OnceLock::new();
    let converter = CONVERTER.get_or_init(|| ConverterBuilder::new().build());
    if s.contains(is_cyrillic) {
        converter.cyr_to_lat(s)
    } else {
        converter.lat_to_cyr(s)
    }
}

fn apply_locale(locale: &str) -> Result<(), String> {
    let cfg_dir = config_dir();
    std::fs::create_dir_all(&cfg_dir).map_err(|e| format!("Cannot create config dir: {}", e))?;

    std::fs::write(cfg_dir.join("locale.conf"), format!("LANG={}\n", locale))
        .map_err(|e| format!("Cannot write locale.conf: {}", e))?;

    let start_marker = "# --- xfce-aero-lang-changer locale ---";
    let end_marker = "# --- end xfce-aero-lang-changer locale ---";
    let locale_export = format!("export LANG={}", locale);

    let xprofile_path = dirs::home_dir().unwrap_or_default().join(".xprofile");
    let mut content = String::new();
    if xprofile_path.exists() {
        content = std::fs::read_to_string(&xprofile_path)
            .map_err(|e| format!("Cannot read ~/.xprofile: {}", e))?;
    }

    let start_idx = content.find(start_marker);
    let end_idx = content.find(end_marker);
    if let (Some(s), Some(e)) = (start_idx, end_idx) {
        let new_section = format!("{}\n{}\n{}\n", start_marker, locale_export, end_marker);
        let after = e + end_marker.len();
        content.replace_range(s..after, &new_section);
    } else {
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&format!("\n{}\n{}\n{}\n", start_marker, locale_export, end_marker));
    }

    std::fs::write(&xprofile_path, content)
        .map_err(|e| format!("Cannot write ~/.xprofile: {}", e))?;

    Ok(())
}

fn logout_xfce() {
    Command::new("xfce4-session-logout").arg("--logout").spawn().ok();
}

fn show_aero_alert(t: &Trans, title: &str, msg: &str) {
    let screen = app::screen_size();
    let dlg_w = 420;
    let dlg_h = 360;
    let mut win = Window::new(
        (screen.0 as i32 - dlg_w) / 2,
        (screen.1 as i32 - dlg_h) / 2,
        dlg_w, dlg_h,
        title,
    );
    win.set_color(Color::White);

    let header_title = title.to_string();
    let mut header = Frame::new(0, 0, dlg_w, 36, "");
    header.set_frame(FrameType::NoBox);
    header.draw(move |f| {
        let w = f.w();
        let h = f.h();
        for y in 0..h {
            let t = y as f64 / h as f64;
            let g = (100u8 as f64 * (1.0 - t) + 148u8 as f64 * t) as u8;
            let b = (92u8 as f64 * (1.0 - t) + 136u8 as f64 * t) as u8;
            fltk::draw::draw_rect_fill(0, y, w, 1, Color::from_rgb(0, g, b));
        }
        let bold = fltk::enums::Font::by_name("sans-serif Bold");
        fltk::draw::set_font(bold, 13);
        fltk::draw::set_draw_color(Color::White);
        fltk::draw::draw_text2(&header_title, 0, 0, w, h, Align::Center | Align::Inside);
    });

    let mut buf = TextBuffer::default();
    buf.set_text(msg);
    let mut text = TextDisplay::new(30, 46, dlg_w - 60, dlg_h - 120, "");
    text.set_buffer(buf);
    text.wrap_mode(WrapMode::AtBounds, 0);
    text.set_text_font(fltk::enums::Font::by_name("sans-serif"));
    text.set_text_size(12);
    text.set_text_color(Color::Black);
    text.set_frame(FrameType::NoBox);

    let mut ok_btn = Button::new((dlg_w - 75) / 2, dlg_h - 46, 75, 26, t.ok.as_str());
    ok_btn.set_label_size(11);
    ok_btn.set_callback(move |_| app::quit());

    win.end();
    win.show();
    let _ = app::run();
}

fn show_aero_msg(t: &Trans, human_name: &str, locale: &str) -> Option<i32> {
    let screen = app::screen_size();
    let dlg_w = 400;
    let dlg_h = 150;
    let title = t.lang_changed_title.as_str();
    let mut win = Window::new(
        (screen.0 as i32 - dlg_w) / 2,
        (screen.1 as i32 - dlg_h) / 2,
        dlg_w, dlg_h,
        title,
    );
    win.set_color(Color::White);
    win.make_modal(true);
    win.set_callback(|_| {});

    let header_label = t.lang_changed_title.clone();
    let mut header = Frame::new(0, 0, dlg_w, 30, "");
    header.set_frame(FrameType::NoBox);
    header.draw(move |f| {
        let w = f.w();
        let h = f.h();
        for y in 0..h {
            let t = y as f64 / h as f64;
            let g = (100u8 as f64 * (1.0 - t) + 148u8 as f64 * t) as u8;
            let b = (92u8 as f64 * (1.0 - t) + 136u8 as f64 * t) as u8;
            fltk::draw::draw_rect_fill(0, y, w, 1, Color::from_rgb(0, g, b));
        }
        let bold = fltk::enums::Font::by_name("sans-serif Bold");
        fltk::draw::set_font(bold, 13);
        fltk::draw::set_draw_color(Color::White);
        fltk::draw::draw_text2(&header_label, 0, 0, w, h, Align::Center | Align::Inside);
    });

    let escaped_locale = locale.replace("@", "@@");
    let msg_text = tr_fmt(&t.lang_changed_msg_fmt, &[("name", human_name), ("locale", &escaped_locale)]);

    let mut msg = Frame::new(15, 38, dlg_w - 30, 64, "");
    msg.set_frame(FrameType::NoBox);
    msg.set_label_size(12);
    msg.set_label_color(Color::Black);
    msg.set_align(Align::Left | Align::Inside);
    msg.set_label(&msg_text);

    let btn_y = dlg_h - 40;
    let mut later_btn = Button::new(dlg_w - 180, btn_y, 75, 24, t.later.as_str());
    later_btn.set_label_size(11);

    let mut logout_btn = Button::new(dlg_w - 95, btn_y, 75, 24, t.logout_btn.as_str());
    logout_btn.set_label_size(11);
    logout_btn.set_label_color(Color::Red);

    let result = Rc::new(RefCell::new(None));
    let ret = result.clone();
    logout_btn.set_callback({
        let ret = ret.clone();
        move |_| {
            *ret.borrow_mut() = Some(0);
            app::quit();
        }
    });
    later_btn.set_callback({
        let ret = ret.clone();
        move |_| {
            *ret.borrow_mut() = Some(1);
            app::quit();
        }
    });

    win.end();
    win.show();
    remove_close_button(title);
    let _ = app::run();

    let ret = result.borrow().clone();
    ret
}

fn remove_close_button(title: &str) {
    if let Ok(out) = Command::new("xdotool")
        .args(["search", "--name", title])
        .output()
    {
        let xid = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !xid.is_empty() {
            let func_mask: u32 = 2 | 4 | 8 | 16;
            Command::new("xprop")
                .args([
                    "-id", &xid,
                    "-f", "_MOTIF_WM_HINTS", "32c",
                    "-set", "_MOTIF_WM_HINTS",
                    &format!("0x1, 0x{:x}, 0x0, 0, 0x0", func_mask),
                ])
                .output()
                .ok();
        }
    }
}

fn is_arch() -> bool {
    std::path::Path::new("/etc/arch-release").exists()
        || std::fs::read_to_string("/etc/os-release")
            .map(|c| c.contains("ID=arch") || c.contains("ID_LIKE=arch"))
            .unwrap_or(false)
}

fn env_check(t: &Trans) -> Vec<String> {
    let mut warnings = Vec::new();

    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    if session_type.eq_ignore_ascii_case("wayland") {
        warnings.push(t.warning_wayland.clone());
    }

    if which::which("gsettings").is_ok() {
        let de = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
        if de.eq_ignore_ascii_case("gnome") || de.contains("GNOME") || de.contains("Budgie") {
            let de_name = if de.contains("Budgie") { "GNOME/Budgie" } else { "GNOME" };
            warnings.push(tr_fmt(&t.warning_gnome_fmt, &[("de", de_name)]));
        }
    }

    if config_dir().join("plasma-localerc").exists() {
        warnings.push(t.warning_kde.clone());
    }

    let dde_env = dirs::home_dir().unwrap_or_default().join(".dde_env");
    if dde_env.exists() {
        warnings.push(t.warning_deepin.clone());
    }

    if !is_arch() {
        warnings.push(t.warning_non_arch.clone());
    }

    warnings
}

fn main() {
    let t = load_translations();

    let theme = WidgetTheme::new(ThemeType::Aero);
    theme.apply();

    let available = Rc::new(RefCell::new(get_available_locales()));
    let current = get_current_locale();

    if available.borrow().is_empty() {
        dialog::alert_default(&t.no_locales);
        std::process::exit(1);
    }

    let warnings = env_check(&t);
    if !warnings.is_empty() {
        let numbered: Vec<String> = warnings.iter().enumerate()
            .map(|(i, w)| format!("{}. {}", i + 1, w))
            .collect();
        let body = numbered.join("\n\n");
        let msg = format!("{}\n\n{}", body, t.compat_quit);
        show_aero_alert(&t, &t.compat_title, &msg);
        std::process::exit(1);
    }

    let screen = app::screen_size();
    let buf_w = 640.min((screen.0 * 0.6) as i32);
    let buf_h = 520.min((screen.1 * 0.6) as i32);

    let mut win = Window::new(
        (screen.0 as i32 - buf_w) / 2,
        (screen.1 as i32 - buf_h) / 2,
        buf_w, buf_h,
        t.window_title.as_str(),
    );
    win.make_resizable(true);

    let header_title = t.header_title.clone();
    let header_subtitle = t.header_subtitle.clone();
    let mut header = Frame::new(0, 0, buf_w, 56, "");
    header.set_frame(FrameType::NoBox);
    header.draw(move |f| {
        let w = f.w();
        let h = f.h();
        for y in 0..h {
            let t = y as f64 / h as f64;
            let r = (0x00u8 as f64 * (1.0 - t) + 0x00u8 as f64 * t) as u8;
            let g = (100u8 as f64 * (1.0 - t) + 148u8 as f64 * t) as u8;
            let b = (92u8 as f64 * (1.0 - t) + 136u8 as f64 * t) as u8;
            fltk::draw::draw_rect_fill(0, y, w, 1, Color::from_rgb(r, g, b));
        }
        let bold = fltk::enums::Font::by_name("sans-serif Bold");
        fltk::draw::set_font(bold, 16);
        fltk::draw::set_draw_color(Color::White);
        fltk::draw::draw_text2(&header_title, 0, 6, w, 24, Align::Center | Align::Inside);
        let reg = fltk::enums::Font::by_name("sans-serif");
        fltk::draw::set_font(reg, 11);
        fltk::draw::set_draw_color(Color::from_hex(0xA0D0CC));
        fltk::draw::draw_text2(&header_subtitle, 0, 32, w, 20, Align::Center | Align::Inside);
        fltk::draw::set_draw_color(AERO_BORDER);
        fltk::draw::draw_rect_fill(0, h - 1, w, 1, AERO_BORDER);
    });

    let body_y = 68;
    let body_h = buf_h - body_y - 8;

    let current_text = tr_fmt(&t.current_fmt, &[("name", &locale_to_human_name(&current)), ("locale", &current)]);
    let mut curr_label = Frame::new(20, body_y, buf_w - 40, 24, "");
    curr_label.set_frame(FrameType::ThinDownBox);
    curr_label.set_color(Color::White);
    curr_label.set_label_size(13);
    curr_label.set_label_color(Color::from_hex(0x09554E));
    curr_label.set_align(Align::Center | Align::Inside);
    curr_label.set_label(&current_text);

    let mut search_label = Frame::new(20, body_y + 25, buf_w - 40, 16, t.search_hint.as_str());
    search_label.set_label_size(10);
    search_label.set_label_color(Color::from_hex(0x888888));
    search_label.set_align(Align::Left | Align::Inside);

    let mut filter_inp = Input::new(20, body_y + 40, buf_w - 40, 23, "");
    filter_inp.set_text_size(11);
    filter_inp.set_text_color(Color::Black);
    filter_inp.set_frame(widget_themes::OS_DEFAULT_BUTTON_UP_BOX);

    let list_y = body_y + 72;
    let list_h = body_h - 104;

    let visible = Rc::new(RefCell::new(Vec::<usize>::new()));
    *visible.borrow_mut() = (0..available.borrow().len()).collect();

    let selected = Rc::new(RefCell::new(-1i32));

    let mut table = TableRow::new(20, list_y, buf_w - 40, list_h, "");
    table.set_table_frame(FrameType::NoBox);
    table.end();
    table.set_rows(visible.borrow().len() as i32);
    table.set_cols(2);
    let col0_w = ((buf_w - 40) as f32 * 0.71) as i32;
    table.set_col_width(0, col0_w);
    table.set_col_width(1, (buf_w - 40) - col0_w - 1);
    table.set_row_resize(false);
    table.set_col_resize(false);
    table.set_row_header(false);
    table.set_col_header(false);

    {
        let avail = available.borrow();
        let vis = visible.borrow();
        for (vi, &idx) in vis.iter().enumerate() {
            if avail[idx].1 == current {
                *selected.borrow_mut() = vi as i32;
                break;
            }
        }
    }

    {
        let avail = available.clone();
        let sel = selected.clone();
        let vis = visible.clone();
        table.draw_cell(move |_t, ctx, row, col, x, y, w, h| {
            match ctx {
                TableContext::StartPage => {
                    fltk::draw::draw_rect_fill(x, y, w, h, Color::White);
                }
                TableContext::EndPage => {
                    fltk::draw::set_draw_color(Color::from_hex(0x9AABB8));
                    fltk::draw::draw_rect(x, y, w, h);
                }
                TableContext::Table | TableContext::RcResize => {
                    fltk::draw::draw_rect_fill(x, y, w, h, Color::White);
                }
                _ => {
                    if row < 0 || col < 0 {
                        return;
                    }
                    let r = row as usize;
                    if r >= vis.borrow().len() {
                        return;
                    }
                    let idx = vis.borrow()[r];
                    if idx >= avail.borrow().len() {
                        return;
                    }
                    let (human, loc) = &avail.borrow()[idx];

                    let is_sel = *sel.borrow() == row;
                    let bg = if is_sel {
                        let t = 0.15f64;
                        let r = (0u8 as f64 * t + 0u8 as f64 * (1.0 - t)) as u8;
                        let g = (100u8 as f64 * t + 148u8 as f64 * (1.0 - t)) as u8;
                        let b = (92u8 as f64 * t + 136u8 as f64 * (1.0 - t)) as u8;
                        Color::from_rgb(r, g, b)
                    } else if r % 2 == 0 {
                        Color::from_hex(0xF0F2F5)
                    } else {
                        Color::White
                    };
                    fltk::draw::draw_rect_fill(x, y, w, h, bg);

                    let font = fltk::enums::Font::by_name("sans-serif");
                    fltk::draw::set_font(font, 13);

                    let fg = if is_sel { Color::White } else { Color::Black };
                    fltk::draw::set_draw_color(fg);

                    let text = if col == 0 { human } else { loc };
                    fltk::draw::draw_text2(text, x + 4, y, w - 4, h, Align::Left | Align::Inside);

                    fltk::draw::set_draw_color(Color::from_hex(0xD8DDE3));
                    fltk::draw::draw_rect_fill(x, y + h - 1, w, 1, Color::from_hex(0xD8DDE3));
                }
            }
        });
    }

    {
        let sel = selected.clone();
        let mut tbl = table.clone();
        let vis = visible.clone();
        table.set_callback(move |_| {
            let row = tbl.callback_row();
            if row >= 0 && (row as usize) < vis.borrow().len() {
                *sel.borrow_mut() = row;
                tbl.redraw();
            }
        });
    }

    let filter_vis = visible.clone();
    let filter_sel = selected.clone();
    let filter_avail = available.clone();
    let mut filter_tbl = table.clone();
    filter_inp.set_trigger(CallbackTrigger::Changed);
    filter_inp.set_callback(move |inp| {
        let q = inp.value().to_lowercase();
        let q_alt = transliterate(&q);
        let mut new_vis = Vec::new();
        let avail = filter_avail.borrow();
        for (i, (human, loc)) in avail.iter().enumerate() {
            let h = human.to_lowercase();
            let l = loc.to_lowercase();
            if q.is_empty() || h.contains(&q) || l.contains(&q) || h.contains(&q_alt) || l.contains(&q_alt) {
                new_vis.push(i);
            }
        }
        *filter_vis.borrow_mut() = new_vis;
        *filter_sel.borrow_mut() = -1;
        filter_tbl.set_rows(filter_vis.borrow().len() as i32);
        filter_tbl.redraw();
    });

    let btn_area_y = list_y + list_h + 8;

    let mut apply_btn = Button::new(buf_w - 100, btn_area_y, 80, 23, t.apply.as_str());
    apply_btn.set_label_size(11);
    apply_btn.set_label_color(Color::Black);
    apply_btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    let mut quit_btn = Button::new(20, btn_area_y, 80, 23, t.quit.as_str());
    quit_btn.set_label_size(11);
    quit_btn.set_label_color(Color::Black);
    quit_btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    win.end();
    win.show();

    let apply_sel = selected.clone();
    let apply_vis = visible.clone();
    let apply_avail = available.clone();
    let dialog_open = Rc::new(RefCell::new(false));
    let apply_dlg_open = dialog_open.clone();
    apply_btn.set_callback(move |_| {
        if *apply_dlg_open.borrow() {
            return;
        }
        let sel = *apply_sel.borrow();
        if sel >= 0 {
            let vis = apply_vis.borrow();
            if (sel as usize) < vis.len() {
                let idx = vis[sel as usize];
                let loc = apply_avail.borrow()[idx].1.clone();
                let name = apply_avail.borrow()[idx].0.clone();
                drop(vis);
                match apply_locale(&loc) {
                    Ok(()) => {
                        *apply_dlg_open.borrow_mut() = true;
                        if show_aero_msg(&t, &name, &loc) == Some(0) {
                            logout_xfce();
                        }
                        *apply_dlg_open.borrow_mut() = false;
                    }
                    Err(e) => dialog::alert_default(&tr_fmt(&t.error_fmt, &[("msg", &e)])),
                }
            }
        }
    });

    quit_btn.set_callback(move |_| app::quit());

    let _ = app::run();
}
