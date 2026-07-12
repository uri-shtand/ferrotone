# cpal 0.15.0 API Summary

## Key Traits
- `DeviceTrait` — `name()`, `supported_input_configs()`, `default_input_config()`, `build_input_stream::<T>(config, data_callback, error_callback, timeout)`, `build_input_stream_raw(config, sample_format, data_callback, error_callback, timeout)`
- `StreamTrait` — `play()`, `pause()`
- `HostTrait` — `devices()`, `default_input_device()`, `default_output_device()`

## Key Types
- `Device` (platform dispatches via DeviceTrait)
- `Stream` (platform dispatches via StreamTrait)
- `StreamConfig { channels: ChannelCount, sample_rate: SampleRate, buffer_size: BufferSize }`
- `SampleRate(f32)` or `SampleRate(u32)` depending on version
- `SupportedStreamConfig` — from `default_input_config()`
- `InputCallbackInfo`, `OutputCallbackInfo`
- `BuildStreamError`, `StreamError`, `DefaultStreamConfigError`, `PlayStreamError`, `PauseStreamError`
- `SampleFormat::F32`, etc.
- `BufferSize::Fixed(u32)` or `BufferSize::Default`
- `Data` — dynamically typed audio buffer

## Usage Pattern
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
let host = cpal::default_host();
let device = host.default_input_device().expect("no input device");
let config = device.default_input_config().unwrap().into();
let stream = device.build_input_stream::<f32, _, _>(&config, move |data, _| { /* data: &[f32] */ }, move |err| {}, None);
stream.play().unwrap();
```
