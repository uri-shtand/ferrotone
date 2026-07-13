import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Settings, DeviceInfo, AudioSettings, NoiseCancellationSettings } from '../types';

const DEFAULT_SETTINGS: Settings = {
  audio: { sample_rate: 48000, buffer_size: 1024, device_name: '', algorithm: 'swipe' },
  noise_cancellation: {
    enabled: false, input_gain: 1.0,
    rms_gate_enabled: true, rms_threshold: 0.01,
    confidence_gate_enabled: true, confidence_threshold: 0.3,
    bandpass_enabled: true, bandpass_low: 80, bandpass_high: 1000,
  },
  user: { cache_folder: '', active_profile: 'default' },
};

interface UseAudioSettingsReturn {
  settings: Settings;
  loading: boolean;
  dirty: boolean;
  updateAudio: (partial: Partial<AudioSettings>) => Promise<void>;
  updateNoiseCancellation: (partial: Partial<NoiseCancellationSettings>) => Promise<void>;
  save: () => Promise<void>;
  devices: DeviceInfo[];
  refreshDevices: () => Promise<void>;
}

export function useAudioSettings(): UseAudioSettingsReturn {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [dirty, setDirty] = useState(false);
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const savedRef = useRef<string | null>(null);

  const serialize = useCallback((s: Settings) => JSON.stringify(s), []);

  useEffect(() => {
    const load = async () => {
      try {
        const s = await invoke<Settings>('get_settings');
        setSettings(s);
        savedRef.current = serialize(s);
        setDirty(false);
      } catch (e) {
        console.error('failed to load settings, using defaults', e);
        savedRef.current = serialize(DEFAULT_SETTINGS);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [serialize]);

  useEffect(() => {
    if (settings && savedRef.current !== null) {
      setDirty(serialize(settings) !== savedRef.current);
    }
  }, [settings, serialize]);

  const refreshDevices = useCallback(async () => {
    try {
      const d = await invoke<DeviceInfo[]>('list_devices');
      setDevices(d);
    } catch (e) {
      console.error('failed to list devices', e);
    }
  }, []);

  useEffect(() => {
    refreshDevices();
  }, [refreshDevices]);

  const updateSettings = useCallback(async (updated: Settings) => {
    setSettings(updated);
    try {
      await invoke('update_settings', { settings: updated });
    } catch (e) {
      console.error('failed to update settings', e);
    }
  }, []);

  const updateAudio = useCallback(
    async (partial: Partial<AudioSettings>) => {
      if (!settings) return;
      const updated: Settings = {
        ...settings,
        audio: { ...settings.audio, ...partial },
      };
      await updateSettings(updated);
    },
    [settings, updateSettings],
  );

  const updateNoiseCancellation = useCallback(
    async (partial: Partial<NoiseCancellationSettings>) => {
      if (!settings) return;
      const updated: Settings = {
        ...settings,
        noise_cancellation: { ...settings.noise_cancellation, ...partial },
      };
      await updateSettings(updated);
    },
    [settings, updateSettings],
  );

  const save = useCallback(async () => {
    try {
      await invoke('save_settings');
      if (settings) {
        savedRef.current = serialize(settings);
        setDirty(false);
      }
    } catch (e) {
      console.error('failed to save settings', e);
    }
  }, [settings, serialize]);

  return {
    settings,
    loading,
    dirty,
    updateAudio,
    updateNoiseCancellation,
    save,
    devices,
    refreshDevices,
  };
}
