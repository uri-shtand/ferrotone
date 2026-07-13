import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import AudioControls from './AudioControls';

const defaultProps = {
  inputGain: 1.0,
  rmsGateEnabled: true,
  rmsThreshold: 0.01,
  confidenceGateEnabled: true,
  confidenceThreshold: 0.3,
  bandpassEnabled: true,
  bandpassLow: 80,
  bandpassHigh: 1000,
  noiseCancellationEnabled: false,
  deviceName: '',
  devices: [{ name: 'Default Mic', is_default: true }],
  onInputGainChange: vi.fn(),
  onRmsGateEnabledChange: vi.fn(),
  onRmsThresholdChange: vi.fn(),
  onConfidenceGateEnabledChange: vi.fn(),
  onConfidenceThresholdChange: vi.fn(),
  onBandpassEnabledChange: vi.fn(),
  onBandpassLowChange: vi.fn(),
  onBandpassHighChange: vi.fn(),
  onNoiseCancellationEnabledChange: vi.fn(),
  onDeviceNameChange: vi.fn(),
  onSave: vi.fn(),
  dirty: false,
};

describe('AudioControls', () => {
  it('renders save button', () => {
    render(<AudioControls {...defaultProps} />);
    expect(screen.getByText('Save Settings')).toBeDefined();
  });

  it('shows dirty state when unsaved', () => {
    render(<AudioControls {...defaultProps} dirty={true} />);
    const btn = screen.getByText('Save Settings') as HTMLButtonElement;
    expect(btn.className).toContain('dirty');
    expect(btn.disabled).toBe(false);
  });

  it('disables save button when clean', () => {
    render(<AudioControls {...defaultProps} dirty={false} />);
    const btn = screen.getByText('Save Settings') as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
  });

  it('calls onSave when save clicked while dirty', () => {
    const onSave = vi.fn();
    render(<AudioControls {...defaultProps} dirty={true} onSave={onSave} />);
    fireEvent.click(screen.getByText('Save Settings'));
    expect(onSave).toHaveBeenCalledOnce();
  });

  it('renders gain slider in expanded Input section', () => {
    render(<AudioControls {...defaultProps} />);
    expect(screen.getByText('1.00x')).toBeDefined();
  });

  it('renders device dropdown with options', () => {
    render(<AudioControls {...defaultProps} />);
    expect(screen.getByText('System Default')).toBeDefined();
    expect(screen.getByText('Default Mic')).toBeDefined();
  });

  it('renders collapsible sections', () => {
    render(<AudioControls {...defaultProps} />);
    expect(screen.getByText('Input')).toBeDefined();
    expect(screen.getByText('Volume Gate')).toBeDefined();
    expect(screen.getByText('Confidence Gate')).toBeDefined();
    expect(screen.getByText('Bandpass Filter')).toBeDefined();
    expect(screen.getByText('Noise Cancellation')).toBeDefined();
  });

  it('shows hint text when clean', () => {
    render(<AudioControls {...defaultProps} dirty={false} />);
    expect(screen.getByText('All changes saved')).toBeDefined();
  });

  it('shows hint text when dirty', () => {
    render(<AudioControls {...defaultProps} dirty={true} />);
    expect(screen.getByText('Unsaved changes')).toBeDefined();
  });
});
