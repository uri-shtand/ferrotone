import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { usePitchGraphCapture } from './usePitchGraphCapture';

const mockListen = vi.fn();

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe('usePitchGraphCapture', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(vi.fn());
  });

  it('listens_to_pitch_frame_event', () => {
    renderHook(() => usePitchGraphCapture(false));
    expect(mockListen).toHaveBeenCalledWith('pitch-frame', expect.any(Function));
  });

  it('buffers_incoming_frames', async () => {
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
      eventCallback = cb;
      return Promise.resolve(vi.fn());
    });

    const { result } = renderHook(() => usePitchGraphCapture(true));

    await act(async () => {
      eventCallback?.({ payload: { frequency_hz: 440, note_name: 'A4', cents_deviation: 0, clarity: 0.9, voiced: true, timestamp_ms: 1000 } });
      eventCallback?.({ payload: { frequency_hz: 523, note_name: 'C5', cents_deviation: -2, clarity: 0.8, voiced: true, timestamp_ms: 1001 } });
    });

    expect(result.current.bufferRef.current).toHaveLength(2);
    expect(result.current.bufferRef.current[0].frequency_hz).toBe(440);
    expect(result.current.bufferRef.current[1].note_name).toBe('C5');
  });

  it('clears_buffer_when_not_capturing', async () => {
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
      eventCallback = cb;
      return Promise.resolve(vi.fn());
    });

    const { result, rerender } = renderHook(
      (capturing: boolean) => usePitchGraphCapture(capturing),
      { initialProps: true },
    );

    await act(async () => {
      eventCallback?.({ payload: { frequency_hz: 440, note_name: 'A4', cents_deviation: 0, clarity: 0.9, voiced: true, timestamp_ms: 1000 } });
    });
    expect(result.current.bufferRef.current).toHaveLength(1);

    rerender(false);
    expect(result.current.bufferRef.current).toHaveLength(0);
  });

  it('prunes_frames_older_than_60s', async () => {
    vi.useFakeTimers();
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
      eventCallback = cb;
      return Promise.resolve(vi.fn());
    });

    const now = Date.now();
    vi.setSystemTime(now);

    const { result } = renderHook(() => usePitchGraphCapture(true));

    await act(async () => {
      eventCallback?.({ payload: { frequency_hz: 440, note_name: 'A4', cents_deviation: 0, clarity: 0.9, voiced: true, timestamp_ms: now - 65_000 } });
      eventCallback?.({ payload: { frequency_hz: 523, note_name: 'C5', cents_deviation: -2, clarity: 0.8, voiced: true, timestamp_ms: now - 5_000 } });
    });

    expect(result.current.bufferRef.current).toHaveLength(2);

    vi.advanceTimersByTime(2000);
    await act(async () => {});
    expect(result.current.bufferRef.current).toHaveLength(1);

    vi.useRealTimers();
  });
});
