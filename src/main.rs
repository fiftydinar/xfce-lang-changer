use std::{cell::RefCell, process::Command, rc::Rc};
use fltk::{
    app, button::Button, dialog,
    enums::{Align, CallbackTrigger, Color, FrameType},
    frame::Frame, input::Input, prelude::*,
    table::{TableContext, TableRow}, window::Window,
};
use fltk_theme::{widget_themes, WidgetTheme, ThemeType};

const AERO_BORDER: Color = Color::from_hex(0x09554E);

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
    let mut raw: Vec<String> = Vec::new();
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

    // Group by base locale (strip encoding suffix), prefer UTF-8
    let mut groups: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for loc in &raw {
        let base = loc.split('.').next().unwrap_or(loc).to_string();
        let is_utf8 = loc.contains(".utf8") || loc.contains(".UTF-8");
        let entry = groups.entry(base);
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

fn lang_name(code: &str) -> &str {
    isolang::Language::from_639_1(code)
        .map(|l| l.to_name())
        .unwrap_or(code)
}

fn country_name(code: &str) -> &str {
    rust_iso3166::from_alpha2(code)
        .map(|c| c.name)
        .unwrap_or(code)
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
    format!("{}{}", lang_name, region_name)
}

fn apply_locale(locale: &str) -> Result<(), String> {
    let config_dir = config_dir();
    std::fs::create_dir_all(&config_dir).map_err(|e| format!("Cannot create config dir: {}", e))?;

    std::fs::write(config_dir.join("locale.conf"), format!("LANG={}\n", locale))
        .map_err(|e| format!("Cannot write locale.conf: {}", e))?;

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
        let bold = fltk::enums::Font::by_name("Noto Sans Bold");
        fltk::draw::set_font(bold, 16);
        fltk::draw::set_draw_color(Color::White);
        fltk::draw::draw_text2("Language Changer", 0, 10, w, 26, Align::Center | Align::Inside);
        let reg = fltk::enums::Font::by_name("Noto Sans");
        fltk::draw::set_font(reg, 11);
        fltk::draw::set_draw_color(Color::from_hex(0xA0D0CC));
        fltk::draw::draw_text2("Change your system language", 0, 32, w, 20, Align::Center | Align::Inside);
        fltk::draw::set_draw_color(AERO_BORDER);
        fltk::draw::draw_rect_fill(0, h - 1, w, 1, AERO_BORDER);
    });

    // ==== Body ====
    let body_y = 68;
    let body_h = buf_h - body_y - 8;

    let mut curr_label = Frame::new(20, body_y, buf_w - 40, 24, "");
    curr_label.set_frame(FrameType::BorderBox);
    curr_label.set_label_size(13);
    curr_label.set_label_color(Color::from_hex(0x09554E));
    curr_label.set_align(Align::Center | Align::Inside);
    curr_label.set_label(&format!("Current: {} - {}", locale_to_human_name(&current), current));

    // Search/filter label
    let mut search_label = Frame::new(20, body_y + 25, buf_w - 40, 16, "Search languages...");
    search_label.set_label_size(10);
    search_label.set_label_color(Color::from_hex(0x888888));
    search_label.set_align(Align::Left | Align::Inside);

    let mut filter_inp = Input::new(20, body_y + 40, buf_w - 40, 23, "");
    filter_inp.set_text_size(11);
    filter_inp.set_text_color(Color::Black);
    filter_inp.set_frame(widget_themes::OS_DEFAULT_BUTTON_UP_BOX);

    // Locale table
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
    let col0_w = ((buf_w - 40) as f32 * 0.55) as i32;
    table.set_col_width(0, col0_w);
    table.set_col_width(1, (buf_w - 40) - col0_w - 1);
    table.set_row_resize(false);
    table.set_col_resize(false);
    table.set_row_header(false);
    table.set_col_header(false);

    // Preselect current locale
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

    // Draw callback
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

                    let font = fltk::enums::Font::by_name("Noto Sans");
                    fltk::draw::set_font(font, 13);

                    let fg = if is_sel { Color::White } else { Color::Black };
                    fltk::draw::set_draw_color(fg);

                    let text = if col == 0 { human } else { loc };
                    let align = if col == 0 { Align::Left } else { Align::Right };
                    let pad = if col == 0 { 4i32 } else { 8i32 };
                    fltk::draw::draw_text2(text, x + pad, y, w - pad * 2, h, align | Align::Inside);

                    fltk::draw::set_draw_color(Color::from_hex(0xD8DDE3));
                    fltk::draw::draw_rect_fill(x, y + h - 1, w, 1, Color::from_hex(0xD8DDE3));
                }
            }
        });
    }

    // Selection callback
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

    // Filter callback
    let filter_vis = visible.clone();
    let filter_sel = selected.clone();
    let filter_avail = available.clone();
    let mut filter_tbl = table.clone();
    filter_inp.set_trigger(CallbackTrigger::Changed);
    filter_inp.set_callback(move |inp| {
        let q = inp.value().to_lowercase();
        let mut new_vis = Vec::new();
        let avail = filter_avail.borrow();
        for (i, (human, loc)) in avail.iter().enumerate() {
            if q.is_empty() || human.to_lowercase().contains(&q) || loc.to_lowercase().contains(&q) {
                new_vis.push(i);
            }
        }
        *filter_vis.borrow_mut() = new_vis;
        *filter_sel.borrow_mut() = -1;
        filter_tbl.set_rows(filter_vis.borrow().len() as i32);
        filter_tbl.redraw();
    });

    // Bottom buttons
    let btn_area_y = list_y + list_h + 8;

    let mut apply_btn = Button::new(buf_w - 100, btn_area_y, 80, 23, "Apply");
    apply_btn.set_label_size(11);
    apply_btn.set_label_color(Color::Black);
    apply_btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    let mut quit_btn = Button::new(20, btn_area_y, 80, 23, "Quit");
    quit_btn.set_label_size(11);
    quit_btn.set_label_color(Color::Black);
    quit_btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    win.end();
    win.show();

    // Apply callback
    let apply_sel = selected.clone();
    let apply_vis = visible.clone();
    let apply_avail = available.clone();
    apply_btn.set_callback(move |_| {
        let vis = apply_vis.borrow();
        let sel = *apply_sel.borrow();
        if sel >= 0 && (sel as usize) < vis.len() {
            let idx = vis[sel as usize];
            let loc = &apply_avail.borrow()[idx].1;
            match apply_locale(loc) {
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
    });

    quit_btn.set_callback(move |_| app::quit());

    app::run().unwrap();
}