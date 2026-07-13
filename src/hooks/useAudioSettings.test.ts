import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { invoke } from '@tauri-apps/api/core';
import { useAudioSettings } from './useAudioSettings';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const mockSettings = {
  audio: {
    sample_rate: 48000,
    buffer_size: 1024,
    device_name: 'Default Mic',
    algorithm: 'swipe',
  },
  noise_cancellation: {
    enabled: false,
    input_gain: 1.0,
    rms_gate_enabled: true,
    rms_threshold: 0.01,
    confidence_gate_enabled: true,
    confidence_threshold: 0.3,
    bandpass_enabled: true,
    bandpass_low: 80,
    bandpass_high: 1000,
  },
  user: {
    cache_folder: '',
    active_profile: 'default',
  },
};

const mockDevices = [
  { name: 'Default Mic', is_default: true },
  { name: 'External Mic', is_default: false },
];

describe('useAudioSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_settings') return mockSettings;
      if (cmd === 'list_devices') return mockDevices;
      return undefined;
    });
  });

  it('loads settings on mount', async () => {
    const { result } = renderHook(() => useAudioSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.settings).toEqual(mockSettings);
    expect(result.current.dirty).toBe(false);
  });

  it('loads devices on mount', async () => {
    const { result } = renderHook(() => useAudioSettings());

    await waitFor(() => {
      expect(result.current.devices.length).toBeGreaterThan(0);
    });

    expect(result.current.devices).toEqual(mockDevices);
  });

  it('updates noise cancellation and marks dirty', async () => {
    const { result } = renderHook(() => useAudioSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.updateNoiseCancellation({ rms_threshold: 0.05 });
    });

    expect(result.current.settings?.noise_cancellation.rms_threshold).toBe(0.05);
    expect(result.current.dirty).toBe(true);
    expect(invoke).toHaveBeenCalledWith('update_settings', {
      settings: expect.objectContaining({
        noise_cancellation: expect.objectContaining({ rms_threshold: 0.05 }),
      }),
    });
  });

  it('updates audio settings and marks dirty', async () => {
    const { result } = renderHook(() => useAudioSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.updateAudio({ device_name: 'External Mic' });
    });

    expect(result.current.settings?.audio.device_name).toBe('External Mic');
    expect(result.current.dirty).toBe(true);
  });

  it('clears dirty on save', async () => {
    const { result } = renderHook(() => useAudioSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.updateNoiseCancellation({ rms_threshold: 0.05 });
    });
    expect(result.current.dirty).toBe(true);

    await act(async () => {
      await result.current.save();
    });

    expect(result.current.dirty).toBe(false);
    expect(invoke).toHaveBeenCalledWith('save_settings');
  });

  it('refreshes devices', async () => {
    const { result } = renderHook(() => useAudioSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const newDevices = [{ name: 'New Mic', is_default: true }];
    vi.mocked(invoke).mockResolvedValueOnce(newDevices);

    await act(async () => {
      await result.current.refreshDevices();
    });

    expect(result.current.devices).toEqual(newDevices);
  });
});
