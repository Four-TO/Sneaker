use crate::settings::Settings;
use crate::state::Shared;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState, Modifiers, Code};

fn parse_shortcut(s: &str) -> Option<Shortcut> {
    let mut mods = Modifiers::empty();
    let mut key_str: Option<&str> = None;
    for part in s.split('+').map(|p| p.trim()) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "meta" | "super" | "win" | "cmd" => mods |= Modifiers::SUPER,
            _ => { key_str = Some(part); }
        }
    }
    let k = key_str?;
    let code = map_code(k)?;
    Some(Shortcut::new(Some(mods), code))
}

fn map_code(k: &str) -> Option<Code> {
    let u = k.to_ascii_uppercase();
    if u.len() == 1 {
        let c = u.chars().next().unwrap();
        return Some(match c {
            'A' => Code::KeyA, 'B' => Code::KeyB, 'C' => Code::KeyC, 'D' => Code::KeyD,
            'E' => Code::KeyE, 'F' => Code::KeyF, 'G' => Code::KeyG, 'H' => Code::KeyH,
            'I' => Code::KeyI, 'J' => Code::KeyJ, 'K' => Code::KeyK, 'L' => Code::KeyL,
            'M' => Code::KeyM, 'N' => Code::KeyN, 'O' => Code::KeyO, 'P' => Code::KeyP,
            'Q' => Code::KeyQ, 'R' => Code::KeyR, 'S' => Code::KeyS, 'T' => Code::KeyT,
            'U' => Code::KeyU, 'V' => Code::KeyV, 'W' => Code::KeyW, 'X' => Code::KeyX,
            'Y' => Code::KeyY, 'Z' => Code::KeyZ,
            '0' => Code::Digit0, '1' => Code::Digit1, '2' => Code::Digit2,
            '3' => Code::Digit3, '4' => Code::Digit4, '5' => Code::Digit5,
            '6' => Code::Digit6, '7' => Code::Digit7, '8' => Code::Digit8,
            '9' => Code::Digit9,
            ',' | '<' => Code::Comma,
            '.' | '>' => Code::Period,
            '/' | '?' => Code::Slash,
            '\\' | '|' => Code::Backslash,
            ';' | ':' => Code::Semicolon,
            '\'' | '"' => Code::Quote,
            '`' | '~' => Code::Backquote,
            '-' | '_' => Code::Minus,
            '=' | '+' => Code::Equal,
            '[' | '{' => Code::BracketLeft,
            ']' | '}' => Code::BracketRight,
            _ => return None,
        });
    }
    Some(match u.as_str() {
        "F1" => Code::F1, "F2" => Code::F2, "F3" => Code::F3, "F4" => Code::F4,
        "F5" => Code::F5, "F6" => Code::F6, "F7" => Code::F7, "F8" => Code::F8,
        "F9" => Code::F9, "F10" => Code::F10, "F11" => Code::F11, "F12" => Code::F12,
        "SPACE" => Code::Space,
        "ENTER" | "RETURN" => Code::Enter,
        "ESC" | "ESCAPE" => Code::Escape,
        "TAB" => Code::Tab,
        "BACKSPACE" => Code::Backspace,
        "DELETE" | "DEL" => Code::Delete,
        "INSERT" | "INS" => Code::Insert,
        "HOME" => Code::Home,
        "END" => Code::End,
        "PAGEUP" => Code::PageUp,
        "PAGEDOWN" => Code::PageDown,
        "ARROWUP" | "UP" => Code::ArrowUp,
        "ARROWDOWN" | "DOWN" => Code::ArrowDown,
        "ARROWLEFT" | "LEFT" => Code::ArrowLeft,
        "ARROWRIGHT" | "RIGHT" => Code::ArrowRight,
        _ => return None,
    })
}

/// Returns list of "field:reason" for failed registrations. Empty = all ok.
pub fn register_all<R: Runtime>(app: &AppHandle<R>, s: &Settings) -> Vec<String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    // Give Windows a moment to actually unhook the hotkeys before re-registering.
    std::thread::sleep(std::time::Duration::from_millis(30));
    let mut failed = Vec::new();
    let specs = [
        (s.hotkeys.toggle_show.clone(), "toggleShow"),
        (s.hotkeys.toggle_top.clone(), "toggleTop"),
        (s.hotkeys.toggle_passthrough.clone(), "togglePassthrough"),
        (s.hotkeys.boss_key.clone(), "bossKey"),
        (s.hotkeys.quick_capture.clone(), "quickCapture"),
        (s.hotkeys.toggle_chrome.clone(), "toggleChrome"),
    ];
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (combo, label) in specs.iter() {
        if combo.is_empty() { continue; }
        if !seen.insert(combo.clone()) {
            failed.push(format!("{}|{}|与其它项重复", label, combo));
            continue;
        }
        match parse_shortcut(combo) {
            Some(sc) => {
                if gs.register(sc).is_err() {
                    failed.push(format!("{}|{}|被占用或无效", label, combo));
                }
            }
            None => failed.push(format!("{}|{}|无法解析", label, combo)),
        }
    }
    failed
}

pub fn on_shortcut<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut) {
    let state: tauri::State<Shared> = app.state();
    let s = state.settings.lock().clone();
    let matches = |combo: &str| -> bool {
        match parse_shortcut(combo) {
            Some(sc) => &sc == shortcut,
            None => false,
        }
    };
    if matches(&s.hotkeys.toggle_show) {
        if let Some(w) = app.get_webview_window("main") {
            let minimized = w.is_minimized().unwrap_or(false);
            let visible = w.is_visible().unwrap_or(false);
            if visible && !minimized { let _ = w.hide(); }
            else { let _ = w.unminimize(); let _ = w.show(); let _ = w.set_focus(); }
        }
    } else if matches(&s.hotkeys.toggle_top) {
        let mut s = state.settings.lock();
        s.always_on_top = !s.always_on_top;
        if let Some(w) = app.get_webview_window("main") { let _ = w.set_always_on_top(s.always_on_top); }
        let _ = crate::storage::save_settings(&s);
        let _ = app.emit("settings-updated", &*s);
    } else if matches(&s.hotkeys.toggle_passthrough) {
        let mut s = state.settings.lock();
        let next = match s.passthrough.as_str() {
            "off" => if s.show_title_bar { "semi" } else { "full" },
            "semi" => "full",
            _ => "off",
        };
        s.passthrough = next.into();
        if let Some(w) = app.get_webview_window("main") {
            crate::commands::apply_passthrough_inner(&w, next);
        }
        let _ = crate::storage::save_settings(&s);
        let _ = app.emit("settings-updated", &*s);
    } else if matches(&s.hotkeys.boss_key) {
        if let Some(w) = app.get_webview_window("main") { let _ = w.hide(); }
        if s.has_master_password {
            *state.locked.lock() = true;
            let _ = app.emit("locked", ());
        }
    } else if matches(&s.hotkeys.toggle_chrome) {
        let mut s = state.settings.lock();
        let target = !(s.show_title_bar || s.show_bottom_bar);
        s.show_title_bar = target;
        s.show_bottom_bar = target;
        if !target && s.passthrough == "semi" { s.passthrough = "off".into(); }
        let _ = crate::storage::save_settings(&s);
        let _ = app.emit("settings-updated", &*s);
    } else if matches(&s.hotkeys.quick_capture) {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.unminimize();
            let _ = w.show();
            let _ = w.set_focus();
        }
        let _ = app.emit("focus-new-task", ());
    }
}

pub fn handle_event<R: Runtime>(app: &AppHandle<R>, shortcut: &Shortcut, event: tauri_plugin_global_shortcut::ShortcutEvent) {
    if event.state() == ShortcutState::Pressed {
        on_shortcut(app, shortcut);
    }
}
