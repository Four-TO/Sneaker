use crate::settings::Settings;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub locked: Mutex<bool>,
}

pub type Shared = Arc<AppState>;

impl AppState {
    pub fn new(settings: Settings) -> Shared {
        let locked = settings.has_master_password;
        Arc::new(Self {
            settings: Mutex::new(settings),
            locked: Mutex::new(locked),
        })
    }
}
