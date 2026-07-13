pub mod capture;
pub mod device;

use serde::{Deserialize, Serialize};

pub use capture::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeFrame {
    pub rms_level: f32,
    pub timestamp_ms: u64,
}
