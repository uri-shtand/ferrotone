use std::sync::Mutex;

use ferrotone_core::audio::CaptureEngine;
use ferrotone_core::config::Settings;

pub struct AppState {
    pub engine: Mutex<Option<CaptureEngine>>,
    pub settings: Mutex<Settings>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            engine: Mutex::new(None),
            settings: Mutex::new(settings),
        }
    }
}
