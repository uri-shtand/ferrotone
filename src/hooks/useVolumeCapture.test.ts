import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useVolumeCapture } from './useVolumeCapture';

const mockListen = vi.fn();

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe('useVolumeCapture', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(vi.fn());
  });

  it('listens_to_volume_frame_event', () => {
    renderHook(() => useVolumeCapture(false));
    expect(mockListen).toHaveBeenCalledWith('volume-frame', expect.any(Function));
  });

  it('buffers_incoming_frames', async () => {
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
      eventCallback = cb;
      return Promise.resolve(vi.fn());
    });

    const { result } = renderHook(() => useVolumeCapture(true));

    await act(async () => {
      eventCallback?.({ payload: { rms_level: 0.5, timestamp_ms: 1000 } });
      eventCallback?.({ payload: { rms_level: 0.3, timestamp_ms: 1001 } });
    });

    expect(result.current.bufferRef.current).toHaveLength(2);
    expect(result.current.bufferRef.current[0].rms_level).toBe(0.5);
    expect(result.current.bufferRef.current[1].rms_level).toBe(0.3);
  });

  it('clears_buffer_when_not_capturing', async () => {
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
      eventCallback = cb;
      return Promise.resolve(vi.fn());
    });

    const { result, rerender } = renderHook(
      (capturing: boolean) => useVolumeCapture(capturing),
      { initialProps: true },
    );

    await act(async () => {
      eventCallback?.({ payload: { rms_level: 0.5, timestamp_ms: 1000 } });
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

    const { result } = renderHook(() => useVolumeCapture(true));

    await act(async () => {
      eventCallback?.({ payload: { rms_level: 0.5, timestamp_ms: now - 65_000 } });
      eventCallback?.({ payload: { rms_level: 0.5, timestamp_ms: now - 5_000 } });
    });

    expect(result.current.bufferRef.current).toHaveLength(2);

    vi.advanceTimersByTime(2000);
    await act(async () => {});
    expect(result.current.bufferRef.current).toHaveLength(1);

    vi.useRealTimers();
  });
});
