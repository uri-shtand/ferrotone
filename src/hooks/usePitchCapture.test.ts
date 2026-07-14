import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { usePitchCapture } from './usePitchCapture';

const mockListen = vi.fn();
const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

describe('usePitchCapture', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockListen.mockResolvedValue(vi.fn());
    mockInvoke.mockResolvedValue(undefined);
  });

  it('listens_to_pitch_frame_event', () => {
    renderHook(() => usePitchCapture());
    expect(mockListen).toHaveBeenCalledWith('pitch-frame', expect.any(Function));
  });

  it('updates_latest_frame_on_event', async () => {
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
      eventCallback = cb;
      return Promise.resolve(vi.fn());
    });

    const { result } = renderHook(() => usePitchCapture());

    const frame = {
      frequency_hz: 440,
      note_name: 'A4',
      cents_deviation: 2.3,
      clarity: 0.94,
      voiced: true,
      timestamp_ms: 1000,
    };

    await act(async () => {
      eventCallback?.({ payload: frame });
    });

    expect(result.current.latestFrame).toEqual(frame);
  });

  it('maintains_history_buffer', async () => {
    let eventCallback: ((event: { payload: unknown }) => void) | null = null;
    mockListen.mockImplementation((_event: string, cb: (event: { payload: unknown }) => void) => {
      eventCallback = cb;
      return Promise.resolve(vi.fn());
    });

    const { result } = renderHook(() => usePitchCapture());

    for (let i = 0; i < 150; i++) {
      await act(async () => {
        eventCallback?.({
          payload: {
            frequency_hz: 440,
            note_name: 'A4',
            cents_deviation: 0,
            clarity: 0.9,
            voiced: true,
            timestamp_ms: i * 10,
          },
        });
      });
    }

    expect(result.current.history.length).toBeLessThanOrEqual(100);
  });

  it('invokes_start_capture_command', async () => {
    const { result } = renderHook(() => usePitchCapture());

    await act(async () => {
      await result.current.start();
    });

    expect(mockInvoke).toHaveBeenCalledWith('start_capture');
    expect(result.current.isCapturing).toBe(true);
  });

  it('invokes_stop_capture_command', async () => {
    const { result } = renderHook(() => usePitchCapture());

    await act(async () => {
      await result.current.start();
    });

    await act(async () => {
      await result.current.stop();
    });

    expect(mockInvoke).toHaveBeenCalledWith('stop_capture');
    expect(result.current.isCapturing).toBe(false);
  });

  it('handles_start_error', async () => {
    mockInvoke.mockRejectedValue('no microphone');

    const { result } = renderHook(() => usePitchCapture());

    await act(async () => {
      await result.current.start();
    });

    expect(result.current.isCapturing).toBe(false);
    expect(result.current.error).toBe('no microphone');
  });
});
