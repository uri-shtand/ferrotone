#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(String),
    #[error("TOML parse error: {0}")]
    Parse(String),
    #[error("TOML serialize error: {0}")]
    Serialize(String),
}

#[derive(Debug, thiserror::Error)]
pub enum DetectionError {
    #[error("pitch detection init failed: {0}")]
    InitFailed(String),
    #[error("no audio device available")]
    NoDevice,
    #[error("unsupported sample rate: {0}")]
    UnsupportedSampleRate(u32),
    #[error("stream error: {0}")]
    StreamError(String),
}

impl From<cpal::BuildStreamError> for DetectionError {
    fn from(e: cpal::BuildStreamError) -> Self {
        DetectionError::StreamError(e.to_string())
    }
}

impl From<cpal::PlayStreamError> for DetectionError {
    fn from(e: cpal::PlayStreamError) -> Self {
        DetectionError::StreamError(e.to_string())
    }
}

impl From<pitch_core::EstimatorError> for DetectionError {
    fn from(e: pitch_core::EstimatorError) -> Self {
        DetectionError::InitFailed(e.to_string())
    }
}
