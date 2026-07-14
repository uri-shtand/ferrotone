import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import PitchDisplay from './PitchDisplay';

describe('PitchDisplay', () => {
  it('renders_idle_state', () => {
    render(
      <PitchDisplay
        isCapturing={false}
        latestFrame={null}
        onStart={vi.fn()}
        onStop={vi.fn()}
      />
    );

    expect(screen.getByText('Idle')).toBeDefined();
    expect(screen.getAllByText('--').length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText(/Start/)).toBeDefined();
  });

  it('renders_pitch_data', () => {
    render(
      <PitchDisplay
        isCapturing={true}
        latestFrame={{
          frequency_hz: 440.0,
          note_name: 'A4',
          cents_deviation: 2.3,
          clarity: 0.94,
          voiced: true,
          timestamp_ms: 1000,
        }}
        onStart={vi.fn()}
        onStop={vi.fn()}
      />
    );

    expect(screen.getByText('A4')).toBeDefined();
    expect(screen.getByText('440.0 Hz')).toBeDefined();
    expect(screen.getByText('+2.3c')).toBeDefined();
    expect(screen.getByText(/94%/)).toBeDefined();
    expect(screen.getByText(/Stop/)).toBeDefined();
    expect(screen.getByText('Listening...')).toBeDefined();
  });

  it('calls_onstart_on_click', () => {
    const onStart = vi.fn();
    render(
      <PitchDisplay
        isCapturing={false}
        latestFrame={null}
        onStart={onStart}
        onStop={vi.fn()}
      />
    );

    fireEvent.click(screen.getByText(/Start/));
    expect(onStart).toHaveBeenCalledTimes(1);
  });

  it('calls_onstop_on_click', () => {
    const onStop = vi.fn();
    render(
      <PitchDisplay
        isCapturing={true}
        latestFrame={{
          frequency_hz: 440,
          note_name: 'A4',
          cents_deviation: 0,
          clarity: 0.9,
          voiced: true,
          timestamp_ms: 0,
        }}
        onStart={vi.fn()}
        onStop={onStop}
      />
    );

    fireEvent.click(screen.getByText(/Stop/));
    expect(onStop).toHaveBeenCalledTimes(1);
  });

  it('shows_stopped_when_not_capturing_with_frame', () => {
    render(
      <PitchDisplay
        isCapturing={false}
        latestFrame={{
          frequency_hz: 440,
          note_name: 'A4',
          cents_deviation: 0,
          clarity: 0.9,
          voiced: true,
          timestamp_ms: 100,
        }}
        onStart={vi.fn()}
        onStop={vi.fn()}
      />
    );

    expect(screen.getByText('Stopped')).toBeDefined();
    expect(screen.getByText(/Start/)).toBeDefined();
  });

  it('shows_negative_cents', () => {
    render(
      <PitchDisplay
        isCapturing={true}
        latestFrame={{
          frequency_hz: 435,
          note_name: 'A4',
          cents_deviation: -19.6,
          clarity: 0.85,
          voiced: true,
          timestamp_ms: 500,
        }}
        onStart={vi.fn()}
        onStop={vi.fn()}
      />
    );

    expect(screen.getByText('-19.6c')).toBeDefined();
  });
});
