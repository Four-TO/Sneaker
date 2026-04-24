use crate::state::Shared;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem, Submenu},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};

const TRAY_ID: &str = "sneaker-tray";

pub fn build_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示主窗", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏主窗", true, None::<&str>)?;
    let top = CheckMenuItem::with_id(app, "top", "置顶", true, false, None::<&str>)?;
    let pt_off = MenuItem::with_id(app, "pt_off", "关闭穿透", true, None::<&str>)?;
    let pt_semi = MenuItem::with_id(app, "pt_semi", "半穿透", true, None::<&str>)?;
    let pt_full = MenuItem::with_id(app, "pt_full", "全穿透", true, None::<&str>)?;
    let pt = Submenu::with_id_and_items(app, "pt", "穿透模式", true, &[&pt_off, &pt_semi, &pt_full])?;
    let settings = MenuItem::with_id(app, "settings", "设置…", true, None::<&str>)?;
    let lock = MenuItem::with_id(app, "lock", "锁定", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &top, &pt, &settings, &lock, &sep, &quit])?;

    let handle = app.clone();
    let _ = TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            tauri::image::Image::new(&[], 0, 0)
        }))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("Sneaker")
        .on_menu_event(move |app, event| handle_menu(app, event.id.as_ref()))
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle().clone();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.unminimize();
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    let _ = handle;
    Ok(())
}

fn handle_menu<R: Runtime>(app: &AppHandle<R>, id: &str) {
    let state: tauri::State<Shared> = app.state();
    match id {
        "show" => { if let Some(w) = app.get_webview_window("main") { let _ = w.unminimize(); let _ = w.show(); let _ = w.set_focus(); } }
        "hide" => { if let Some(w) = app.get_webview_window("main") { let _ = w.hide(); } }
        "top" => {
            let mut s = state.settings.lock();
            s.always_on_top = !s.always_on_top;
            if let Some(w) = app.get_webview_window("main") { let _ = w.set_always_on_top(s.always_on_top); }
            let _ = crate::storage::save_settings(&s);
            let _ = app.emit("settings-updated", &*s);
        }
        "pt_off" => set_pt(app, "off"),
        "pt_semi" => set_pt(app, "semi"),
        "pt_full" => set_pt(app, "full"),
        "settings" => {
            if let Some(w) = app.get_webview_window("main") { let _ = w.unminimize(); let _ = w.show(); let _ = w.set_focus(); }
            let _ = app.emit("view-change", "settings");
        }
        "lock" => {
            let s = state.settings.lock();
            if s.has_master_password {
                drop(s);
                *state.locked.lock() = true;
                let _ = app.emit("locked", ());
            }
        }
        "quit" => { app.exit(0); }
        _ => {}
    }
}

fn set_pt<R: Runtime>(app: &AppHandle<R>, mode: &str) {
    let state: tauri::State<Shared> = app.state();
    {
        let mut s = state.settings.lock();
        s.passthrough = mode.into();
        let _ = crate::storage::save_settings(&s);
    }
    if let Some(w) = app.get_webview_window("main") {
        crate::commands::apply_passthrough_inner(&w, mode);
    }
    let _ = app.emit("settings-updated", &*state.settings.lock());
}

pub fn apply_tray_visibility<R: Runtime>(app: &AppHandle<R>, visible: bool) {
    if visible {
        if app.tray_by_id(TRAY_ID).is_none() {
            let _ = build_tray(app);
        }
    } else {
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let _ = app.remove_tray_by_id(TRAY_ID);
            let _ = tray;
        }
    }
}
