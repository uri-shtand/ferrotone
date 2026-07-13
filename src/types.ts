export interface PitchFrame {
  frequency_hz: number;
  note_name: string;
  cents_deviation: number;
  clarity: number;
  timestamp_ms: number;
}

export interface CaptureStatus {
  isCapturing: boolean;
  error: string | null;
}

export interface AudioSettings {
  sample_rate: number;
  buffer_size: number;
  device_name: string;
  algorithm: string;
}

export interface NoiseCancellationSettings {
  enabled: boolean;
  input_gain: number;
  rms_gate_enabled: boolean;
  rms_threshold: number;
  confidence_gate_enabled: boolean;
  confidence_threshold: number;
  bandpass_enabled: boolean;
  bandpass_low: number;
  bandpass_high: number;
}

export interface UserSettings {
  cache_folder: string;
  active_profile: string;
}

export interface Settings {
  audio: AudioSettings;
  noise_cancellation: NoiseCancellationSettings;
  user: UserSettings;
}

export interface DeviceInfo {
  name: string;
  is_default: boolean;
}
