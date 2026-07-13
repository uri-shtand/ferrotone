import { useEffect, useRef } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { VolumeFrame } from '../types';

const WINDOW_MS = 60_000;

interface UseVolumeCaptureReturn {
  bufferRef: React.MutableRefObject<VolumeFrame[]>;
}

export function useVolumeCapture(isCapturing: boolean): UseVolumeCaptureReturn {
  const bufferRef = useRef<VolumeFrame[]>([]);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    const setup = async () => {
      unlistenRef.current = await listen<VolumeFrame>('volume-frame', (event) => {
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
