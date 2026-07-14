import { useEffect, useRef } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { PitchFrame } from '../types';

const WINDOW_MS = 60_000;

interface UsePitchGraphCaptureReturn {
  bufferRef: React.MutableRefObject<PitchFrame[]>;
}

export function usePitchGraphCapture(isCapturing: boolean): UsePitchGraphCaptureReturn {
  const bufferRef = useRef<PitchFrame[]>([]);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    const setup = async () => {
      unlistenRef.current = await listen<PitchFrame>('pitch-frame', (event) => {
        bufferRef.current.push(event.payload);
      });
    };
    setup();
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  useEffect(() => {
    if (!isCapturing) {
      bufferRef.current = [];
    }
  }, [isCapturing]);

  useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now();
      bufferRef.current = bufferRef.current.filter(
        (f) => now - f.timestamp_ms < WINDOW_MS,
      );
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  return { bufferRef };
}
