use serde::{Deserialize, Serialize};

fn default_drag_mod() -> String { "Alt".into() }
fn default_true() -> bool { true }
fn default_toggle_chrome() -> String { "Ctrl+Alt+B".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hotkeys {
    pub toggle_show: String,
    pub toggle_top: String,
    pub toggle_passthrough: String,
    pub boss_key: String,
    pub quick_capture: String,
    #[serde(default = "default_toggle_chrome")]
    pub toggle_chrome: String,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            toggle_show: "Ctrl+Alt+N".into(),
            toggle_top: "Ctrl+Alt+T".into(),
            toggle_passthrough: "Ctrl+Alt+P".into(),
            boss_key: "Ctrl+Alt+H".into(),
            quick_capture: "Ctrl+Alt+Q".into(),
            toggle_chrome: "Ctrl+Alt+B".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub theme: String,
    pub opacity: f64,
    pub bg_color: String,
    pub blur: String,
    pub always_on_top: bool,
    pub show_title_bar: bool,
    pub show_tray_icon: bool,
    pub skip_taskbar: bool,
    pub passthrough: String,
    pub autostart: bool,
    pub auto_lock_minutes: u32,
    pub has_master_password: bool,
    pub hotkeys: Hotkeys,
    pub notes_dir: String,
    #[serde(default)]
    pub show_sidebar: bool,
    #[serde(default = "default_true")]
    pub show_bottom_bar: bool,
    #[serde(default)]
    pub transparent_bg: bool,
    #[serde(default = "default_drag_mod")]
    pub drag_modifier: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub master_salt: Option<String>,
    #[serde(default)]
    pub window_x: Option<i32>,
    #[serde(default)]
    pub window_y: Option<i32>,
    #[serde(default)]
    pub window_w: Option<u32>,
    #[serde(default)]
    pub window_h: Option<u32>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            opacity: 0.88,
            bg_color: "#14161e".into(),
            blur: "acrylic".into(),
            always_on_top: false,
            show_title_bar: true,
            show_tray_icon: true,
            skip_taskbar: false,
            passthrough: "off".into(),
            autostart: false,
            auto_lock_minutes: 0,
            has_master_password: false,
            hotkeys: Hotkeys::default(),
            notes_dir: String::new(),
            show_sidebar: false,
            show_bottom_bar: true,
            transparent_bg: false,
            drag_modifier: "Alt".into(),
            master_hash: None,
            master_salt: None,
            window_x: None,
            window_y: None,
            window_w: None,
            window_h: None,
        }
    }
}
