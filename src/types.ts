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
