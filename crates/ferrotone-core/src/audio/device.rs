use serde::Serialize;
use cpal::traits::{DeviceTrait, HostTrait};

use crate::audio::capture::CaptureConfig;
use crate::error::DetectionError;

#[derive(Debug, Clone, Serialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
}

pub fn list_input_devices(default_name: &str) -> Result<Vec<AudioDeviceInfo>, String> {
    let host = cpal::default_host();
    let devices = host.devices().map_err(|e| format!("failed to enumerate devices: {e}"))?;

    let mut result = Vec::new();
    for device in devices {
        let name = device.name().unwrap_or_else(|_| "(unknown)".into());
        let is_default = name == default_name || default_name.is_empty();
        result.push(AudioDeviceInfo { name, is_default });
    }

    Ok(result)
}

pub(crate) fn log_device_enumeration(host: &cpal::Host) {
    match host.devices() {
        Ok(devices) => {
            for device in devices {
                let name = device.name().unwrap_or_else(|_| "(unknown)".into());
                tracing::debug!(device = %name, "found input device");
                if let Ok(configs) = device.supported_input_configs() {
                    for cfg in configs {
                        tracing::trace!(
                            device = %name,
                            channels = cfg.channels(),
                            min_sample_rate = cfg.min_sample_rate().0,
                            max_sample_rate = cfg.max_sample_rate().0,
                            buffer_size = ?cfg.buffer_size(),
                            "supported input config"
                        );
                    }
                }
            }
        }
        Err(e) => tracing::warn!(error = %e, "failed to enumerate audio devices"),
    }
}

pub(crate) fn resolve_device(
    host: &cpal::Host,
    config: &CaptureConfig,
) -> Result<cpal::Device, DetectionError> {
    let device = if let Some(ref name) = config.device_name {
        let found = host
            .devices()
            .ok()
            .into_iter()
            .flatten()
            .find(|d| d.name().ok().as_deref() == Some(name.as_str()));
        if found.is_some() {
            tracing::info!(device = %name, "using specified input device");
        } else {
            tracing::warn!(device = %name, "specified device not found, trying default");
        }
        found
    } else {
        let default = host.default_input_device();
        if let Some(ref dev) = default {
            let name = dev.name().unwrap_or_else(|_| "(unknown)".into());
            tracing::info!(device = %name, "using default input device");
        } else {
            tracing::error!("no default input device found");
        }
        default
    };
    device.ok_or(DetectionError::NoDevice)
}

pub(crate) fn resolve_input_config(
    device: &cpal::Device,
    desired: &CaptureConfig,
) -> Result<cpal::StreamConfig, DetectionError> {
    let device_name = device.name().unwrap_or_else(|_| "(unknown)".into());

    let default_cfg = device.default_input_config().map_err(|e| {
        tracing::error!(device = %device_name, error = %e, "no default input config");
        DetectionError::StreamError(format!("no default input config for {device_name}: {e}"))
    })?;

    tracing::debug!(
        device = %device_name,
        default_channels = default_cfg.channels(),
        default_sample_rate = default_cfg.sample_rate().0,
        "device default input config"
    );

    let supported: Vec<_> = match device.supported_input_configs() {
        Ok(cfgs) => {
            let collected: Vec<_> = cfgs.collect();
            for cfg in &collected {
                tracing::debug!(
                    device = %device_name,
                    channels = cfg.channels(),
                    min_sample_rate = cfg.min_sample_rate().0,
                    max_sample_rate = cfg.max_sample_rate().0,
                    buffer_size = ?cfg.buffer_size(),
                    "supported input config"
                );
            }
            collected
        }
        Err(e) => {
            tracing::warn!(device = %device_name, error = %e, "cannot enumerate supported configs, using default");
            return Ok(cpal::StreamConfig {
                channels: default_cfg.channels(),
                sample_rate: default_cfg.sample_rate(),
                buffer_size: cpal::BufferSize::Default,
            });
        }
    };

    if supported.is_empty() {
        tracing::warn!(device = %device_name, "no supported input configs, using default");
        return Ok(cpal::StreamConfig {
            channels: default_cfg.channels(),
            sample_rate: default_cfg.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        });
    }

    let desired_rate = desired.sample_rate;

    let exact_mono = supported.iter().find(|cfg| {
        cfg.channels() == 1
            && desired_rate >= cfg.min_sample_rate().0
            && desired_rate <= cfg.max_sample_rate().0
    });

    if let Some(cfg) = exact_mono {
        let rate = desired_rate.clamp(cfg.min_sample_rate().0, cfg.max_sample_rate().0);
        tracing::info!(
            device = %device_name,
            requested_rate = desired_rate,
            resolved_rate = rate,
            channels = 1u32,
            "using mono config"
        );
        return Ok(cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(rate),
            buffer_size: cpal::BufferSize::Default,
        });
    }

    let default_channels = default_cfg.channels();
    let best_rate = supported
        .iter()
        .filter(|cfg| cfg.channels() == default_channels)
        .flat_map(|cfg| {
            let low = cfg.min_sample_rate().0;
            let high = cfg.max_sample_rate().0;
            if desired_rate >= low && desired_rate <= high {
                vec![desired_rate]
            } else {
                vec![low, high]
            }
        })
        .min_by_key(|&r| (desired_rate as i32 - r as i32).unsigned_abs())
        .unwrap_or(default_cfg.sample_rate().0);

    tracing::info!(
        device = %device_name,
        requested_rate = desired_rate,
        resolved_rate = best_rate,
        channels = default_channels,
        note = "mono not available, using device channel count"
    );

    Ok(cpal::StreamConfig {
        channels: default_channels,
        sample_rate: cpal::SampleRate(best_rate),
        buffer_size: cpal::BufferSize::Default,
    })
}
