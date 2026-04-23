mod settings;
mod storage;
mod state;
mod commands;
mod tray;
mod hotkeys;
mod win_util;

use state::AppState;
#[allow(unused_imports)]
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let loaded = storage::load_settings();
    let shared = AppState::new(loaded.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    hotkeys::handle_event(app, shortcut, event);
                })
                .build(),
        )
        .manage(shared.clone())
        .invoke_handler(tauri::generate_handler![
            commands::load_settings,
            commands::save_settings,
            commands::apply_window_settings,
            commands::apply_hotkeys,
            commands::apply_theme,
            commands::set_opacity,
            commands::set_always_on_top,
            commands::set_passthrough,
            commands::set_title_bar,
            commands::set_tray_visible,
            commands::set_skip_taskbar,
            commands::set_autostart,
            commands::list_notes,
            commands::read_note,
            commands::save_note,
            commands::create_note,
            commands::delete_note,
            commands::rename_note,
            commands::import_file,
            commands::set_note_encryption,
            commands::set_master_password,
            commands::unlock_app,
            commands::lock_app,
            commands::is_locked,
            commands::boss_hide,
            commands::toggle_main,
            commands::pause_hotkeys,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            let s = shared.settings.lock().clone();

            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_always_on_top(s.always_on_top);
                let _ = w.set_skip_taskbar(s.skip_taskbar);
                #[cfg(windows)]
                if let Ok(h) = w.hwnd() {
                    win_util::apply_tool_window(h.0 as isize, s.skip_taskbar);
                }
                commands::apply_passthrough_inner(&w, &s.passthrough);
                commands::apply_blur(&w, &s.blur);
            }

            if s.show_tray_icon {
                let _ = tray::build_tray(&handle);
            }

            hotkeys::register_all(&handle, &s);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
