import { useState } from 'react';
import type { DeviceInfo } from '../types';

interface AudioControlsProps {
  inputGain: number;
  rmsGateEnabled: boolean;
  rmsThreshold: number;
  confidenceGateEnabled: boolean;
  confidenceThreshold: number;
  bandpassEnabled: boolean;
  bandpassLow: number;
  bandpassHigh: number;
  noiseCancellationEnabled: boolean;
  rnnoiseEnabled: boolean;
  deviceName: string;
  devices: DeviceInfo[];
  onInputGainChange: (v: number) => void;
  onRmsGateEnabledChange: (v: boolean) => void;
  onRmsThresholdChange: (v: number) => void;
  onConfidenceGateEnabledChange: (v: boolean) => void;
  onConfidenceThresholdChange: (v: number) => void;
  onBandpassEnabledChange: (v: boolean) => void;
  onBandpassLowChange: (v: number) => void;
  onBandpassHighChange: (v: number) => void;
  onNoiseCancellationEnabledChange: (v: boolean) => void;
  onRnnoiseEnabledChange: (v: boolean) => void;
  onDeviceNameChange: (v: string) => void;
  onSave: () => void;
  dirty: boolean;
}

function Section({
  label,
  open,
  onToggle,
  children,
}: {
  label: string;
  open: boolean;
  onToggle: () => void;
  children: React.ReactNode;
}) {
  return (
    <div className="audio-section">
      <button className="audio-section-header" onClick={onToggle} type="button">
        <span className={`audio-section-arrow ${open ? 'open' : ''}`}>&#9662;</span>
        <span>{label}</span>
      </button>
      {open && <div className="audio-section-body">{children}</div>}
    </div>
  );
}

function Toggle({
  value,
  onChange,
  label,
}: {
  value: boolean;
  onChange: (v: boolean) => void;
  label: string;
}) {
  return (
    <div className="audio-toggle-row">
      <span className="audio-toggle-label">{label}</span>
      <button
        className={`audio-toggle ${value ? 'on' : 'off'}`}
        onClick={() => onChange(!value)}
        type="button"
      >
        {value ? 'ON' : 'OFF'}
      </button>
    </div>
  );
}

function Slider({
  value,
  onChange,
  label,
  min,
  max,
  step,
  display,
}: {
  value: number;
  onChange: (v: number) => void;
  label: string;
  min: number;
  max: number;
  step: number;
  display: string;
}) {
  return (
    <div className="audio-slider-row">
      <span className="audio-slider-label">{label}</span>
      <input
        type="range"
        className="audio-slider"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
      />
      <span className="audio-slider-value">{display}</span>
    </div>
  );
}

export default function AudioControls({
  inputGain,
  rmsGateEnabled,
  rmsThreshold,
  confidenceGateEnabled,
  confidenceThreshold,
  bandpassEnabled,
  bandpassLow,
  bandpassHigh,
  noiseCancellationEnabled,
  rnnoiseEnabled,
  deviceName,
  devices,
  onInputGainChange,
  onRmsGateEnabledChange,
  onRmsThresholdChange,
  onConfidenceGateEnabledChange,
  onConfidenceThresholdChange,
  onBandpassEnabledChange,
  onBandpassLowChange,
  onBandpassHighChange,
  onNoiseCancellationEnabledChange,
  onRnnoiseEnabledChange,
  onDeviceNameChange,
  onSave,
  dirty,
}: AudioControlsProps) {
  const [openSections, setOpenSections] = useState<Record<string, boolean>>({
    input: true,
    nc: false,
    rms: false,
    confidence: false,
    bandpass: false,
  });

  const toggle = (key: string) =>
    setOpenSections((prev) => ({ ...prev, [key]: !prev[key] }));

  return (
    <div className="audio-controls">
      <Section
        label="Input"
        open={openSections.input}
        onToggle={() => toggle('input')}
      >
        <Slider
          label="Gain"
          value={inputGain}
          onChange={onInputGainChange}
          min={0}
          max={2}
          step={0.05}
          display={`${inputGain.toFixed(2)}x`}
        />
        {devices.length > 0 && (
          <div className="audio-slider-row">
            <span className="audio-slider-label">Device</span>
            <select
              className="audio-select"
              value={deviceName}
              onChange={(e) => onDeviceNameChange(e.target.value)}
            >
              <option value="">System Default</option>
              {devices.map((d) => (
                <option key={d.name} value={d.name}>
                  {d.name}
                </option>
              ))}
            </select>
          </div>
        )}
      </Section>

      <Section
        label="Volume Gate"
        open={openSections.rms}
        onToggle={() => toggle('rms')}
      >
        <Toggle
          label="Enabled"
          value={rmsGateEnabled}
          onChange={onRmsGateEnabledChange}
        />
        <Slider
          label="Threshold"
          value={rmsThreshold}
          onChange={onRmsThresholdChange}
          min={0}
          max={0.5}
          step={0.001}
          display={rmsThreshold.toFixed(3)}
        />
      </Section>

      <Section
        label="Confidence Gate"
        open={openSections.confidence}
        onToggle={() => toggle('confidence')}
      >
        <Toggle
          label="Enabled"
          value={confidenceGateEnabled}
          onChange={onConfidenceGateEnabledChange}
        />
        <Slider
          label="Threshold"
          value={confidenceThreshold}
          onChange={onConfidenceThresholdChange}
          min={0}
          max={1}
          step={0.01}
          display={confidenceThreshold.toFixed(2)}
        />
      </Section>

      <Section
        label="Noise Cancellation"
        open={openSections.nc}
        onToggle={() => toggle('nc')}
      >
        <Toggle
          label="Master"
          value={noiseCancellationEnabled}
          onChange={onNoiseCancellationEnabledChange}
        />
        <Toggle
          label="RNNoise"
          value={rnnoiseEnabled}
          onChange={onRnnoiseEnabledChange}
        />
      </Section>

      <Section
        label="Bandpass Filter"
        open={openSections.bandpass}
        onToggle={() => toggle('bandpass')}
      >
        <Toggle
          label="Enabled"
          value={bandpassEnabled}
          onChange={onBandpassEnabledChange}
        />
        <Slider
          label="Low Cut"
          value={bandpassLow}
          onChange={onBandpassLowChange}
          min={20}
          max={2000}
          step={1}
          display={`${bandpassLow.toFixed(0)} Hz`}
        />
        <Slider
          label="High Cut"
          value={bandpassHigh}
          onChange={onBandpassHighChange}
          min={100}
          max={4000}
          step={1}
          display={`${bandpassHigh.toFixed(0)} Hz`}
        />
      </Section>

      <div className="audio-controls-footer">
        <button
          className={`btn btn-save ${dirty ? 'dirty' : ''}`}
          onClick={onSave}
          disabled={!dirty}
          type="button"
        >
          Save Settings
        </button>
        <span className="audio-hint">
          {dirty ? 'Unsaved changes' : 'All changes saved'}
        </span>
      </div>
    </div>
  );
}
