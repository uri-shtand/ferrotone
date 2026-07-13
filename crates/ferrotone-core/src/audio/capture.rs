use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};

use cpal::traits::{DeviceTrait, StreamTrait};

use crate::audio::device;
use crate::error::DetectionError;
use crate::gate::{ConfidenceGate, RmsGate};
use crate::pitch::{PitchDetector, PitchFrame};

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

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub device_name: Option<String>,
    pub rms_gate_enabled: bool,
    pub rms_threshold: f32,
    pub confidence_gate_enabled: bool,
    pub confidence_threshold: f32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 1024,
            device_name: None,
            rms_gate_enabled: true,
            rms_threshold: 0.01,
            confidence_gate_enabled: true,
            confidence_threshold: 0.3,
        }
    }
}

enum ControlSignal {
    Stop,
}

pub struct CaptureEngine {
    detector: Box<dyn PitchDetector>,
    config: CaptureConfig,
    rms_gate: RmsGate,
    confidence_gate: ConfidenceGate,
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
            rms_gate: RmsGate::new(config.rms_threshold)
                .with_enabled(config.rms_gate_enabled),
            confidence_gate: ConfidenceGate::new(config.confidence_threshold)
                .with_enabled(config.confidence_gate_enabled),
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

        device::log_device_enumeration(&host);

        let device = device::resolve_device(&host, &self.config)?;
        let device_name = device.name().unwrap_or_else(|_| "(unknown)".into());

        let stream_config = device::resolve_input_config(&device, &self.config)?;
        tracing::info!(
            device = %device_name,
            sample_rate = stream_config.sample_rate.0,
            channels = stream_config.channels,
            buffer_size = ?stream_config.buffer_size,
            "building input stream"
        );

        let (cpal_stream, raw_rx) = self.build_input_stream(&device, &stream_config)?;

        let (control_tx, control_rx) = bounded::<ControlSignal>(16);
        self.control_tx = control_tx;

        let handle = self.spawn_dsp_worker(raw_rx, control_rx)?;

        self.handle = Some(handle);
        self.stream = SafeStream(Some(cpal_stream));
        self.running.store(true, Ordering::SeqCst);

        Ok(())
    }

    fn build_input_stream(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Result<(cpal::Stream, Receiver<Vec<f32>>), DetectionError> {
        let device_name = device.name().unwrap_or_else(|_| "(unknown)".into());
        let channels = config.channels;
        let (raw_tx, raw_rx) = bounded::<Vec<f32>>(256);

        let error_callback = move |err: cpal::StreamError| {
            tracing::error!(error = %err, "cpal stream error");
        };

        let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mono = if channels >= 2 {
                data.iter().step_by(channels as usize).copied().collect()
            } else {
                data.to_vec()
            };
            if let Err(e) = raw_tx.try_send(mono) {
                tracing::warn!(error = %e, "dropped audio buffer");
            }
        };

        let stream = device
            .build_input_stream::<f32, _, _>(config, data_callback, error_callback, None)
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
        stream.play().inspect_err(|e| {
            tracing::error!(error = %e, "failed to start stream");
        })?;
        tracing::info!("capture stream is now live");

        Ok((stream, raw_rx))
    }

    fn spawn_dsp_worker(
        &mut self,
        raw_rx: Receiver<Vec<f32>>,
        control_rx: Receiver<ControlSignal>,
    ) -> Result<std::thread::JoinHandle<()>, DetectionError> {
        let pitch_tx = self.pitch_tx.clone();
        let mut detector = std::mem::replace(
            &mut self.detector,
            Box::new(crate::pitch::dummy::DummyDetector::new(0.0, 0.0, false)),
        );
        let buffer_size = self.config.buffer_size;
        let rms_gate = std::mem::replace(
            &mut self.rms_gate,
            RmsGate::new(0.01).with_enabled(false),
        );
        let confidence_gate = std::mem::replace(
            &mut self.confidence_gate,
            ConfidenceGate::new(0.3).with_enabled(false),
        );

        let handle = std::thread::Builder::new()
            .name("dsp-worker".into())
            .spawn(move || {
                let mut buffer = Vec::with_capacity(buffer_size);

                loop {
                    match control_rx.try_recv() {
                        Ok(ControlSignal::Stop)
                        | Err(crossbeam_channel::TryRecvError::Disconnected) => {
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
                                if !rms_gate.process(&chunk) {
                                    continue;
                                }
                                let frames = detector.process(&chunk);
                                let frames = confidence_gate.process(frames);
                                for frame in frames {
                                    let _ = pitch_tx.send(frame);
                                }
                            }
                        }
                        Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                        Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
                    }
                }

                if !buffer.is_empty() && rms_gate.process(&buffer) {
                    let frames = detector.process(&buffer);
                    let frames = confidence_gate.process(frames);
                    for frame in frames {
                        let _ = pitch_tx.send(frame);
                    }
                }
            })
            .map_err(|e| DetectionError::StreamError(e.to_string()))?;

        Ok(handle)
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

    pub fn feed_audio(&mut self, samples: &[f32]) -> Vec<PitchFrame> {
        if !self.rms_gate.process(samples) {
            return Vec::new();
        }
        let frames = self.detector.process(samples);
        let frames = self.confidence_gate.process(frames);
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
