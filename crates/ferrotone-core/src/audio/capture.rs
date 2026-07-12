use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};

/// Safe wrapper around `cpal::Stream` which is not `Send` on Windows.
/// This is safe because we only access the stream through a `Mutex` from a single thread at a time.
#[allow(dead_code)]
pub(crate) struct SafeStream(Option<cpal::Stream>);
unsafe impl Send for SafeStream {}

#[allow(dead_code)]
impl SafeStream {
    pub fn new(stream: cpal::Stream) -> Self {
        Self(Some(stream))
    }

    pub fn take(&mut self) -> Option<cpal::Stream> {
        self.0.take()
    }
}

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::DetectionError;
use crate::pitch::{PitchDetector, PitchFrame};

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub device_name: Option<String>,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 1024,
            device_name: None,
        }
    }
}

enum ControlSignal {
    Stop,
}

pub struct CaptureEngine {
    detector: Box<dyn PitchDetector>,
    config: CaptureConfig,
    pitch_tx: Sender<PitchFrame>,
    pitch_rx: Receiver<PitchFrame>,
    control_tx: Sender<ControlSignal>,
    handle: Option<std::thread::JoinHandle<()>>,
    stream: SafeStream,
    running: Arc<AtomicBool>,
}

impl CaptureEngine {
    pub fn new(detector: Box<dyn PitchDetector>, config: CaptureConfig) -> Self {
        let (pitch_tx, pitch_rx) = bounded(64);
        let (control_tx, _) = bounded(16);
        Self {
            detector,
            config,
            pitch_tx,
            pitch_rx,
            control_tx,
            handle: None,
            stream: SafeStream(None),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) -> Result<(), DetectionError> {
        if self.running.load(Ordering::SeqCst) {
            return Err(DetectionError::StreamError(
                "capture already running".into(),
            ));
        }

        tracing::info!("starting capture engine");

        let host = cpal::default_host();
        tracing::info!(host_id = ?host.id(), "using audio host");

        // Enumerate all available devices and their supported configs
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

        let device = if let Some(ref name) = self.config.device_name {
            let found = host.devices()
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
        }
        .ok_or(DetectionError::NoDevice)?;

        let device_name = device.name().unwrap_or_else(|_| "(unknown)".into());

        // Resolve stream config: prefer requested settings but fall back
        let config = resolve_input_config(&device, &self.config)?;
        tracing::info!(
            device = %device_name,
            sample_rate = config.sample_rate.0,
            channels = config.channels,
            buffer_size = ?config.buffer_size,
            "building input stream"
        );

        let channels = config.channels;
        let (raw_tx, raw_rx) = bounded::<Vec<f32>>(256);
        let running = self.running.clone();

        let error_callback = move |err: cpal::StreamError| {
            tracing::error!(error = %err, "cpal stream error");
        };

        let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mono = if channels >= 2 {
                // Interleaved stereo — take only the first channel
                data.iter().step_by(channels as usize).copied().collect()
            } else {
                data.to_vec()
            };
            if let Err(e) = raw_tx.try_send(mono) {
                tracing::warn!(error = %e, "dropped audio buffer");
            }
        };

        let cpal_stream = device
            .build_input_stream::<f32, _, _>(&config, data_callback, error_callback, None)
            .inspect_err(|e| {
                tracing::error!(
                    device = %device_name,
                    sample_rate = config.sample_rate.0,
                    channels = config.channels,
                    buffer_size = ?config.buffer_size,
                    error = %e,
                    "failed to build input stream"
                );
            })?;

        tracing::info!("starting stream playback");
        cpal_stream.play().inspect_err(|e| {
            tracing::error!(error = %e, "failed to start stream");
        })?;
        tracing::info!("capture stream is now live");

        let (control_tx, control_rx) = bounded::<ControlSignal>(16);
        self.control_tx = control_tx;

        let pitch_tx = self.pitch_tx.clone();
        let mut detector = std::mem::replace(
            &mut self.detector,
            Box::new(crate::pitch::dummy::DummyDetector::new(0.0, 0.0, false)),
        );
        let buffer_size = self.config.buffer_size;

        let running_clone = running.clone();
        running_clone.store(true, Ordering::SeqCst);

        let handle = std::thread::Builder::new()
            .name("dsp-worker".into())
            .spawn(move || {
                let mut buffer = Vec::with_capacity(buffer_size);

                loop {
                    match control_rx.try_recv() {
                        Ok(ControlSignal::Stop) | Err(crossbeam_channel::TryRecvError::Disconnected) => {
                            break;
                        }
                        _ => {}
                    }

                    match raw_rx.recv_timeout(Duration::from_millis(10)) {
                        Ok(samples) => {
                            buffer.extend_from_slice(&samples);

                            let chunk_size = buffer_size.min(buffer.len());
                            if chunk_size >= buffer_size / 2 {
                                let chunk: Vec<f32> = buffer.drain(..chunk_size).collect();
                                let frames = detector.process(&chunk);
                                for frame in frames {
                                    let _ = pitch_tx.send(frame);
                                }
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    }
                }

                // Process any remaining samples
                if !buffer.is_empty() {
                    let frames = detector.process(&buffer);
                    for frame in frames {
                        let _ = pitch_tx.send(frame);
                    }
                }
            })
            .map_err(|e| DetectionError::StreamError(e.to_string()))?;

        self.handle = Some(handle);
        self.stream = SafeStream(Some(cpal_stream));
        self.running.store(true, Ordering::SeqCst);

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), DetectionError> {
        if !self.running.load(Ordering::SeqCst) {
            tracing::debug!("stop called but engine not running");
            return Ok(());
        }

        tracing::info!("stopping capture engine");
        self.running.store(false, Ordering::SeqCst);
        let _ = self.control_tx.send(ControlSignal::Stop);

        if let Some(handle) = self.handle.take() {
            tracing::debug!("joining dsp-worker thread");
            let _ = handle.join();
        }

        self.stream.0.take();
        tracing::info!("capture engine stopped");
        Ok(())
    }

    pub fn pitch_receiver(&self) -> &Receiver<PitchFrame> {
        &self.pitch_rx
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Feed audio samples directly into the detector pipeline without cpal.
    /// Used for testing — pushes resulting PitchFrames into the same pitch channel.
    pub fn feed_audio(&mut self, samples: &[f32]) -> Vec<PitchFrame> {
        let frames = self.detector.process(samples);
        for frame in &frames {
            let _ = self.pitch_tx.send(frame.clone());
        }
        frames
    }

    pub fn pitch_sender(&self) -> Sender<PitchFrame> {
        self.pitch_tx.clone()
    }
}

impl Drop for CaptureEngine {
    fn drop(&mut self) {
        tracing::debug!("dropping CaptureEngine");
        let _ = self.stop();
    }
}

fn resolve_input_config(
    device: &cpal::Device,
    desired: &CaptureConfig,
) -> Result<cpal::StreamConfig, DetectionError> {
    let device_name = device.name().unwrap_or_else(|_| "(unknown)".into());

    // Start with the device's default config
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

    // Enumerate supported configs and log them
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

    // Strategy:
    // 1. Prefer exact match: desired channels + rate if supported
    // 2. Fall back to device's default config (channels + rate)
    // 3. Use the device's exact channel count — WASAPI requires exact match

    let desired_rate = desired.sample_rate;

    // Does the device support mono at the desired rate?
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

    // Mono not available — use the default config's channel count and find
    // the best supported rate. We'll extract mono from the first channel.
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
