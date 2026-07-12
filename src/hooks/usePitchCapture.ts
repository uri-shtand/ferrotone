import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { PitchFrame } from '../types';

interface UsePitchCaptureReturn {
  isCapturing: boolean;
  error: string | null;
  latestFrame: PitchFrame | null;
  history: PitchFrame[];
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

const MAX_HISTORY = 100;

export function usePitchCapture(): UsePitchCaptureReturn {
  const [isCapturing, setIsCapturing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [latestFrame, setLatestFrame] = useState<PitchFrame | null>(null);
  const [history, setHistory] = useState<PitchFrame[]>([]);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    const setup = async () => {
      unlistenRef.current = await listen<PitchFrame>('pitch-frame', (event) => {
        setLatestFrame(event.payload);
        setHistory((prev) => {
          const next = [...prev, event.payload];
          return next.length > MAX_HISTORY ? next.slice(-MAX_HISTORY) : next;
        });
      });
    };
    setup();
    return () => {
      unlistenRef.current?.();
    };
  }, []);

  const start = useCallback(async () => {
    try {
      setError(null);
      await invoke('start_capture');
      setIsCapturing(true);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const stop = useCallback(async () => {
    try {
      await invoke('stop_capture');
      setIsCapturing(false);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return { isCapturing, error, latestFrame, history, start, stop };
}
