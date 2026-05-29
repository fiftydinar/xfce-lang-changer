use std::{cell::RefCell, process::Command, rc::Rc};
use fltk::{
    app, browser::HoldBrowser, button::Button, dialog,
    enums::{Align, CallbackTrigger, Color, FrameType},
    frame::Frame, input::Input, prelude::*, window::Window,
};
use fltk_theme::{widget_themes, WidgetTheme, ThemeType};

extern "C" {
    fn Fl_Check_Browser_set_text_font(self_: *mut std::ffi::c_void, f: i32);
    fn Fl_Check_Browser_set_text_color(self_: *mut std::ffi::c_void, c: u32);
}

const AERO_BORDER: Color = Color::from_hex(0x09554E);
const AERO_HEADER_SUBTITLE: Color = Color::from_hex(0xFFFFFF);

fn config_dir() -> std::path::PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        std::path::PathBuf::from(xdg)
    } else {
        let home = std::env::var("HOME").unwrap_or_default();
        std::path::PathBuf::from(home).join(".config")
    }
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
    let mut raw = Vec::new();
    if let Ok(output) = Command::new("locale").arg("-a").output() {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let l = line.trim();
                if !l.is_empty() && l != "C" && l != "POSIX" && l != "C.utf8" {
                    raw.push(l.to_string());
                }
            }
        }
    }

    raw.sort();
    raw.into_iter().map(|loc| {
        let human = locale_to_human_name(&loc);
        (human, loc)
    }).collect()
}

fn lang_name(code: &str) -> &str {
    match code {
        "en" => "English", "de" => "Deutsch", "fr" => "Français",
        "es" => "Español", "it" => "Italiano", "pt" => "Português",
        "ru" => "Русский", "ja" => "日本語", "zh" => "中文",
        "ko" => "한국어", "ar" => "العربية", "nl" => "Nederlands",
        "pl" => "Polski", "tr" => "Türkçe", "sv" => "Svenska",
        "da" => "Dansk", "fi" => "Suomi", "nb" => "Norsk",
        "cs" => "Čeština", "sk" => "Slovenčina", "hu" => "Magyar",
        "ro" => "Română", "bg" => "Български", "uk" => "Українська",
        "el" => "Ελληνικά", "he" => "עברית", "hi" => "हिन्दी",
        "th" => "ไทย", "vi" => "Tiếng Việt", "id" => "Bahasa Indonesia",
        "ms" => "Bahasa Melayu", "bn" => "বাংলা", "ta" => "தமிழ்",
        "te" => "తెలుగు", "mr" => "मराठी", "gu" => "ગુજરાતી",
        "kn" => "ಕನ್ನಡ", "ml" => "മലയാളം", "si" => "සිංහල",
        "ne" => "नेपाली", "fa" => "فارسی", "ur" => "اردو",
        "ca" => "Català", "eu" => "Euskara", "gl" => "Galego",
        "hr" => "Hrvatski", "sr" => "Српски", "sl" => "Slovenščina",
        "lt" => "Lietuvių", "lv" => "Latviešu", "et" => "Eesti",
        "sq" => "Shqip", "mk" => "Македонски", "bs" => "Bosanski",
        "is" => "Íslenska", "ga" => "Gaeilge", "cy" => "Cymraeg",
        "mt" => "Malti", "af" => "Afrikaans", "sw" => "Kiswahili",
        "am" => "ᠠᠮᠠᠷᠢ", "my" => "မြန်မာဘာသာ", "km" => "ភាសាខ្មែរ",
        "lo" => "ລາວ", "mn" => "Монгол", "bo" => "བོད་སྐད",
        "kk" => "Қазақ", "ky" => "Кыргыз", "tg" => "Тоҷикӣ",
        "tk" => "Türkmen", "uz" => "Oʻzbek", "hy" => "Հայերեն",
        "ka" => "ქართული", "ps" => "پښتو", "sd" => "سنڌي",
        "ug" => "ئۇيغۇرچە", "tt" => "Татар",
        _ => code,
    }
}

fn country_name(code: &str) -> &str {
    match code {
        "US" => "United States", "GB" => "UK", "DE" => "Germany",
        "FR" => "France", "ES" => "Spain", "IT" => "Italy",
        "PT" => "Portugal", "BR" => "Brazil", "RU" => "Russia",
        "JP" => "Japan", "CN" => "China", "KR" => "South Korea",
        "SA" => "Saudi Arabia", "AE" => "UAE", "NL" => "Netherlands",
        "PL" => "Poland", "TR" => "Turkey", "SE" => "Sweden",
        "DK" => "Denmark", "FI" => "Finland", "NO" => "Norway",
        "CZ" => "Czechia", "SK" => "Slovakia", "HU" => "Hungary",
        "RO" => "Romania", "BG" => "Bulgaria", "GR" => "Greece",
        "IL" => "Israel", "IN" => "India", "TH" => "Thailand",
        "VN" => "Vietnam", "ID" => "Indonesia", "MY" => "Malaysia",
        "BD" => "Bangladesh", "HK" => "Hong Kong", "TW" => "Taiwan",
        "SG" => "Singapore", "CH" => "Switzerland", "AT" => "Austria",
        "BE" => "Belgium", "CA" => "Canada", "AU" => "Australia",
        "NZ" => "New Zealand", "ZA" => "South Africa", "MX" => "Mexico",
        "AR" => "Argentina", "CL" => "Chile", "CO" => "Colombia",
        "PE" => "Peru", "UA" => "Ukraine", "EG" => "Egypt",
        "MA" => "Morocco", "NG" => "Nigeria", "KE" => "Kenya",
        "PH" => "Philippines", "PK" => "Pakistan", "IR" => "Iran",
        "IQ" => "Iraq", "DZ" => "Algeria", "TN" => "Tunisia",
        _ => code,
    }
}

fn locale_to_human_name(locale: &str) -> String {
    let lang_part = locale.split('.').next().unwrap_or(locale);
    let parts: Vec<&str> = lang_part.split('_').collect();
    let lang_code = parts[0];
    let region_code = parts.get(1).copied().unwrap_or("");

    let lang_name = lang_name(lang_code);
    let region_name = if !region_code.is_empty() {
        let c = country_name(region_code);
        format!(" ({})", c)
    } else {
        String::new()
    };
    format!("{}{}  —  {}", lang_name, region_name, locale)
}

fn apply_locale(locale: &str) -> Result<(), String> {
    let config_dir = config_dir();
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("Cannot create config dir: {}", e))?;

    std::fs::write(config_dir.join("locale.conf"), format!("LANG={}\n", locale))
        .map_err(|e| format!("Cannot write locale.conf: {}", e))?;

    let env_dir = config_dir.join("environment.d");
    std::fs::create_dir_all(&env_dir).map_err(|e| format!("Cannot create environment.d: {}", e))?;
    std::fs::write(env_dir.join("99-lang.conf"), format!("LANG={}\n", locale))
        .map_err(|e| format!("Cannot write environment config: {}", e))?;

    let xfce4_dir = config_dir.join("xfce4");
    std::fs::create_dir_all(&xfce4_dir).ok();
    let env_path = xfce4_dir.join("environment");
    let mut env_content = String::new();
    if let Ok(existing) = std::fs::read_to_string(&env_path) {
        for line in existing.lines() {
            if !line.starts_with("LANG=") {
                env_content.push_str(line);
                env_content.push('\n');
            }
        }
    }
    env_content.push_str(&format!("LANG={}\n", locale));
    std::fs::write(&env_path, env_content).ok();

    Command::new("systemctl")
        .args(["--user", "set-environment", &format!("LANG={}", locale)])
        .output().ok();

    Ok(())
}

fn logout_xfce() {
    Command::new("xfce4-session-logout").arg("--logout").spawn().ok();
}

fn main() {
    let theme = WidgetTheme::new(ThemeType::Aero);
    theme.apply();

    let available = Rc::new(RefCell::new(get_available_locales()));
    let current = get_current_locale();

    if available.borrow().is_empty() {
        dialog::alert_default(
            "No locales found!\n\n\
              Run 'sudo locale-gen' to generate locales,\n\
              or the system image may not have them.\n\n\
              See: cat /etc/locale.gen",
        );
        return;
    }

    let screen = app::screen_size();
    let buf_w = 640.min((screen.0 * 0.6) as i32);
    let buf_h = 520.min((screen.1 * 0.6) as i32);

    let mut win = Window::new(
        (screen.0 as i32 - buf_w) / 2,
        (screen.1 as i32 - buf_h) / 2,
        buf_w, buf_h,
        "Language Changer",
    );
    win.make_resizable(true);

    // ==== Aero-style header bar ====
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
        fltk::draw::set_draw_color(Color::White);
        fltk::draw::draw_text("Language Changer", 20, 22);
        fltk::draw::set_draw_color(AERO_HEADER_SUBTITLE);
        fltk::draw::draw_text("Change your system language", 20, 40);
        fltk::draw::draw_rect_fill(0, h - 1, w, 1, AERO_BORDER);
    });

    // ==== Body ====
    let body_y = 68;
    let body_h = buf_h - body_y - 8;

    let mut curr_label = Frame::new(20, body_y, buf_w - 40, 24, "");
    curr_label.set_label_size(13);
    curr_label.set_label_color(Color::Black);
    curr_label.set_align(Align::Left | Align::Inside);
    curr_label.set_label(&format!("Current: {}", locale_to_human_name(&current)));

    // Search/filter label
    let mut search_label = Frame::new(20, body_y + 25, buf_w - 40, 16, "Search languages...");
    search_label.set_label_size(10);
    search_label.set_label_color(Color::from_hex(0x888888));
    search_label.set_align(Align::Left | Align::Inside);

    let mut filter_inp = Input::new(20, body_y + 40, buf_w - 115, 23, "");
    filter_inp.set_text_size(11);
    filter_inp.set_text_color(Color::Black);
    filter_inp.set_frame(widget_themes::OS_DEFAULT_BUTTON_UP_BOX);

    let mut refresh_btn = Button::new(buf_w - 85, body_y + 40, 65, 23, "Refresh");
    refresh_btn.set_label_size(11);
    refresh_btn.set_label_color(Color::Black);
    refresh_btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    // Locale browser list
    let list_y = body_y + 72;
    let list_h = body_h - 104;
    let mut browser = HoldBrowser::new(20, list_y, buf_w - 40, list_h, "");
    browser.set_text_size(11);
    // Use a font with Arabic support to prevent RTL text overlap
    unsafe {
        let font = fltk::enums::Font::by_name("Noto Sans");
        Fl_Check_Browser_set_text_font(browser.as_widget_ptr() as *mut _, font.bits());
        Fl_Check_Browser_set_text_color(browser.as_widget_ptr() as *mut _, Color::Black.bits());
    }
    for (human, _) in available.borrow().iter() {
        browser.add(human);
    }

    // Preselect current locale
    for (i, (_, loc)) in available.borrow().iter().enumerate() {
        if loc == &current {
            browser.select((i + 1) as i32);
            break;
        }
    }

    // Filter callback
    let mut filter_bro = browser.clone();
    let filter_avail = available.clone();
    filter_inp.set_trigger(CallbackTrigger::Changed);
    filter_inp.set_callback(move |inp| {
        let q = inp.value().to_lowercase();
        filter_bro.clear();
        for (human, _) in filter_avail.borrow().iter() {
            if q.is_empty() || human.to_lowercase().contains(&q) {
                filter_bro.add(human);
            }
        }
    });

    // Bottom buttons
    let btn_area_y = list_y + list_h + 8;

    let mut apply_btn = Button::new(buf_w - 100, btn_area_y, 80, 23, "Apply");
    apply_btn.set_label_size(11);
    apply_btn.set_label_color(Color::Black);
    apply_btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    let mut quit_btn = Button::new(20, btn_area_y, 55, 23, "Quit");
    quit_btn.set_label_size(11);
    quit_btn.set_label_color(Color::Black);
    quit_btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    win.end();
    win.show();

    let apply_bro = browser.clone();
    let apply_avail = available.clone();
    apply_btn.set_callback(move |_| {
        let idx = apply_bro.value();
        if idx > 0 {
            if let Some(sel) = apply_bro.text(idx) {
                let loc = apply_avail.borrow().iter().find(|(h, _)| h == &sel).map(|(_, l)| l.clone());
                if let Some(loc) = loc {
                    match apply_locale(&loc) {
                        Ok(()) => {
                            let msg = format!(
                                "Language set to:\n{}\n\nChanges will apply fully after logout.\n\nLog out now?",
                                loc
                            );
                            if dialog::choice2_default(&msg, "Log Out", "Later", "") == Some(0) {
                                logout_xfce();
                            }
                        }
                        Err(e) => dialog::alert_default(&format!("Error: {}", e)),
                    }
                }
            }
        }
    });

    quit_btn.set_callback(move |_| app::quit());

    let mut ref_bro = browser.clone();
    let ref_avail = available.clone();
    let mut ref_label = curr_label.clone();
    let ref_inp = filter_inp.clone();
    refresh_btn.set_callback(move |_| {
        let new_list = get_available_locales();
        *ref_avail.borrow_mut() = new_list;
        ref_bro.clear();
        let q = ref_inp.value().to_lowercase();
        for (human, _) in ref_avail.borrow().iter() {
            if q.is_empty() || human.to_lowercase().contains(&q) {
                ref_bro.add(human);
            }
        }
        let new_current = get_current_locale();
        ref_label.set_label(&format!("Current: {}", locale_to_human_name(&new_current)));
    });

    app::run().unwrap();
}