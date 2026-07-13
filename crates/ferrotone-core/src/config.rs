use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub audio: AudioSettings,
    pub noise_cancellation: NoiseCancellationSettings,
    pub user: UserSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub device_name: String,
    pub algorithm: String,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 1024,
            device_name: String::new(),
            algorithm: "swipe".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseCancellationSettings {
    pub enabled: bool,
    pub rms_gate_enabled: bool,
    pub rms_threshold: f32,
    pub confidence_gate_enabled: bool,
    pub confidence_threshold: f32,
    pub bandpass_enabled: bool,
    pub bandpass_low: f32,
    pub bandpass_high: f32,
}

impl Default for NoiseCancellationSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            rms_gate_enabled: true,
            rms_threshold: 0.01,
            confidence_gate_enabled: true,
            confidence_threshold: 0.3,
            bandpass_enabled: true,
            bandpass_low: 80.0,
            bandpass_high: 1000.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub cache_folder: String,
    pub active_profile: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            cache_folder: String::new(),
            active_profile: "default".into(),
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();

        if path.exists() {
            let content =
                std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(e.to_string()))?;
            let settings: Settings =
                toml::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))?;
            tracing::info!("settings loaded from {}", path.display());
            Ok(settings)
        } else {
            tracing::info!(
                "no config file at {}, creating defaults",
                path.display()
            );
            let settings = Settings::default();
            settings.save()?;
            Ok(settings)
        }
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ConfigError::Io(e.to_string()))?;
        }
        let content =
            toml::to_string_pretty(self).map_err(|e| ConfigError::Serialize(e.to_string()))?;
        std::fs::write(&path, &content).map_err(|e| ConfigError::Io(e.to_string()))?;
        tracing::info!("settings saved to {}", path.display());
        Ok(())
    }

    fn config_path() -> PathBuf {
        let cwd_path = std::env::current_dir()
            .unwrap_or_default()
            .join("config.toml");

        if cfg!(debug_assertions) || cwd_path.exists() {
            cwd_path
        } else {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("ferrotone")
                .join("config.toml")
        }
    }
}
