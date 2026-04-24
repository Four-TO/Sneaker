use crate::settings::Settings;
use crate::state::Shared;
use crate::storage;
use crate::tasks;
use crate::tray;
use crate::hotkeys;
use crate::win_util;
use tauri::{AppHandle, Emitter, Manager, Runtime, State, WebviewWindow};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

fn emit_settings<R: Runtime>(app: &AppHandle<R>, s: &Settings) {
    let _ = app.emit("settings-updated", s);
}

fn get_main<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    app.get_webview_window("main")
}

#[tauri::command]
pub fn load_settings(state: State<Shared>) -> Settings {
    state.settings.lock().clone()
}

#[tauri::command]
pub fn save_settings(state: State<Shared>, settings: Settings) -> Result<(), String> {
    {
        let mut s = state.settings.lock();
        *s = settings.clone();
    }
    storage::save_settings(&settings).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn apply_window_settings<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, settings: Settings) -> Result<(), String> {
    {
        let mut s = state.settings.lock();
        *s = settings.clone();
    }
    let _ = storage::save_settings(&settings);
    if let Some(w) = get_main(&app) {
        let _ = w.set_always_on_top(settings.always_on_top);
        let _ = w.set_skip_taskbar(settings.skip_taskbar);
        #[cfg(windows)]
        if let Ok(h) = w.hwnd() {
            win_util::apply_tool_window(h.0 as isize, settings.skip_taskbar);
        }
        apply_passthrough_inner(&w, &settings.passthrough);
        apply_blur(&w, &settings.blur);
    }
    tray::apply_tray_visibility(&app, settings.show_tray_icon);
    hotkeys::register_all(&app, &settings);
    Ok(())
}

#[tauri::command]
pub async fn apply_hotkeys<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, settings: Settings) -> Result<bool, String> {
    {
        let mut s = state.settings.lock();
        *s = settings.clone();
    }
    let _ = storage::save_settings(&settings);
    Ok(hotkeys::register_all(&app, &settings))
}

#[tauri::command]
pub async fn apply_theme<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, settings: Settings) -> Result<(), String> {
    {
        let mut s = state.settings.lock();
        *s = settings.clone();
    }
    let _ = storage::save_settings(&settings);
    if let Some(w) = get_main(&app) {
        apply_blur(&w, &settings.blur);
    }
    Ok(())
}

pub fn apply_blur<R: Runtime>(w: &WebviewWindow<R>, blur: &str) {
    #[cfg(windows)]
    {
        use window_vibrancy::{apply_acrylic, apply_mica, clear_acrylic, clear_mica};
        let _ = clear_acrylic(w);
        let _ = clear_mica(w);
        match blur {
            "acrylic" => { let _ = apply_acrylic(w, Some((18, 18, 28, 160))); },
            "mica" => { let _ = apply_mica(w, Some(true)); },
            _ => {}
        }
    }
}

#[tauri::command]
pub async fn set_opacity<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, value: f64) -> Result<(), String> {
    state.settings.lock().opacity = value;
    let _ = storage::save_settings(&state.settings.lock());
    let _ = app;
    Ok(())
}

#[tauri::command]
pub async fn set_always_on_top<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, value: bool) -> Result<(), String> {
    state.settings.lock().always_on_top = value;
    if let Some(w) = get_main(&app) { let _ = w.set_always_on_top(value); }
    let _ = storage::save_settings(&state.settings.lock());
    Ok(())
}

pub fn apply_passthrough_inner<R: Runtime>(w: &WebviewWindow<R>, mode: &str) {
    match mode {
        "full" => {
            let _ = w.set_ignore_cursor_events(true);
            #[cfg(windows)]
            if let Ok(h) = w.hwnd() { win_util::apply_click_through(h.0 as isize, true); }
        }
        "semi" => {
            let _ = w.set_ignore_cursor_events(false);
            #[cfg(windows)]
            if let Ok(h) = w.hwnd() { win_util::apply_click_through(h.0 as isize, false); }
        }
        _ => {
            let _ = w.set_ignore_cursor_events(false);
            #[cfg(windows)]
            if let Ok(h) = w.hwnd() { win_util::apply_click_through(h.0 as isize, false); }
        }
    }
}

#[tauri::command]
pub async fn set_passthrough<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, mode: String) -> Result<(), String> {
    state.settings.lock().passthrough = mode.clone();
    if let Some(w) = get_main(&app) { apply_passthrough_inner(&w, &mode); }
    let _ = storage::save_settings(&state.settings.lock());
    Ok(())
}

#[tauri::command]
pub async fn set_title_bar<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, value: bool) -> Result<(), String> {
    state.settings.lock().show_title_bar = value;
    if !value {
        let cur = state.settings.lock().passthrough.clone();
        if cur == "semi" { state.settings.lock().passthrough = "off".into(); }
    }
    emit_settings(&app, &state.settings.lock());
    let _ = storage::save_settings(&state.settings.lock());
    Ok(())
}

#[tauri::command]
pub async fn set_tray_visible<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, value: bool) -> Result<(), String> {
    state.settings.lock().show_tray_icon = value;
    tray::apply_tray_visibility(&app, value);
    let _ = storage::save_settings(&state.settings.lock());
    Ok(())
}

#[tauri::command]
pub async fn set_skip_taskbar<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, value: bool) -> Result<(), String> {
    state.settings.lock().skip_taskbar = value;
    if let Some(w) = get_main(&app) {
        let _ = w.set_skip_taskbar(value);
        #[cfg(windows)]
        if let Ok(h) = w.hwnd() {
            win_util::apply_tool_window(h.0 as isize, value);
        }
    }
    let _ = storage::save_settings(&state.settings.lock());
    Ok(())
}

#[tauri::command]
pub async fn set_autostart<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>, value: bool) -> Result<(), String> {
    state.settings.lock().autostart = value;
    use tauri_plugin_autostart::ManagerExt;
    let mgr = app.autolaunch();
    let r = if value { mgr.enable() } else { mgr.disable() };
    r.map_err(|e| e.to_string())?;
    let _ = storage::save_settings(&state.settings.lock());
    Ok(())
}

// Notes
#[tauri::command]
pub fn list_notes(state: State<Shared>) -> Result<Vec<storage::NoteMeta>, String> {
    storage::list_notes(&state.settings.lock()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn read_note(state: State<Shared>, id: String, password: Option<String>) -> Result<String, String> {
    storage::read_note(&state.settings.lock(), &id, password.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_note(state: State<Shared>, id: String, content: String, password: Option<String>) -> Result<(), String> {
    storage::save_note(&state.settings.lock(), &id, &content, password.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_note(state: State<Shared>, name: String) -> Result<storage::NoteMeta, String> {
    storage::create_note(&state.settings.lock(), &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(state: State<Shared>, id: String) -> Result<(), String> {
    storage::delete_note(&state.settings.lock(), &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_note(state: State<Shared>, id: String, new_name: String) -> Result<storage::NoteMeta, String> {
    storage::rename_note(&state.settings.lock(), &id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_file(state: State<Shared>, path: String) -> Result<storage::NoteMeta, String> {
    storage::import_file(&state.settings.lock(), &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_note_encryption(state: State<Shared>, id: String, encrypt: bool, password: String) -> Result<(), String> {
    storage::set_note_encryption(&state.settings.lock(), &id, encrypt, &password).map_err(|e| e.to_string())
}

// Security
#[tauri::command]
pub fn set_master_password(state: State<Shared>, old_password: String, new_password: String) -> Result<(), String> {
    let mut s = state.settings.lock();
    if s.has_master_password {
        let hash = s.master_hash.clone().unwrap_or_default();
        let salt = s.master_salt.clone().unwrap_or_default();
        let ok = storage::verify_password(&old_password, &hash, &salt).map_err(|e| e.to_string())?;
        if !ok { return Err("当前密码错误".into()); }
    }
    if new_password.is_empty() {
        s.has_master_password = false;
        s.master_hash = None;
        s.master_salt = None;
    } else {
        let (h, sl) = storage::hash_password(&new_password).map_err(|e| e.to_string())?;
        s.has_master_password = true;
        s.master_hash = Some(h);
        s.master_salt = Some(sl);
    }
    storage::save_settings(&s).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn unlock_app(state: State<Shared>, password: String) -> Result<bool, String> {
    let s = state.settings.lock();
    if !s.has_master_password { *state.locked.lock() = false; return Ok(true); }
    let hash = s.master_hash.clone().unwrap_or_default();
    let salt = s.master_salt.clone().unwrap_or_default();
    let ok = storage::verify_password(&password, &hash, &salt).map_err(|e| e.to_string())?;
    if ok { *state.locked.lock() = false; }
    Ok(ok)
}

#[tauri::command]
pub fn lock_app<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>) -> Result<(), String> {
    let s = state.settings.lock();
    if !s.has_master_password { return Ok(()); }
    *state.locked.lock() = true;
    let _ = app.emit("locked", ());
    Ok(())
}

#[tauri::command]
pub fn is_locked(state: State<Shared>) -> bool {
    *state.locked.lock()
}

#[tauri::command]
pub fn boss_hide<R: Runtime>(app: AppHandle<R>, state: State<'_, Shared>) -> Result<(), String> {
    if let Some(w) = get_main(&app) { let _ = w.hide(); }
    let s = state.settings.lock();
    if s.has_master_password {
        drop(s);
        *state.locked.lock() = true;
        let _ = app.emit("locked", ());
    }
    Ok(())
}

// Tasks
#[tauri::command]
pub fn list_tasks() -> Vec<tasks::Task> { tasks::load_all() }

#[tauri::command]
pub fn create_task(title: String) -> Result<tasks::Task, String> {
    tasks::create(&title).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task(id: String, patch: tasks::TaskPatch) -> Result<tasks::Task, String> {
    tasks::update(&id, patch).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(id: String) -> Result<(), String> {
    tasks::delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_task(id: String) -> Result<tasks::Task, String> {
    tasks::toggle(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_hotkeys<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let _ = app.global_shortcut().unregister_all();
    Ok(())
}

#[tauri::command]
pub fn toggle_main<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    if let Some(w) = get_main(&app) {
        let minimized = w.is_minimized().unwrap_or(false);
        let visible = w.is_visible().unwrap_or(false);
        if visible && !minimized {
            let _ = w.hide();
        } else {
            let _ = w.unminimize();
            let _ = w.show();
            let _ = w.set_focus();
        }
    }
    Ok(())
}
