import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import PitchGraph from './PitchGraph';

describe('PitchGraph', () => {
  it('renders_canvas', () => {
    const bufferRef = { current: [] };
    const { container } = render(
      <PitchGraph bufferRef={bufferRef} isCapturing={false} />,
    );
    const canvas = container.querySelector('canvas');
    expect(canvas).toBeDefined();
  });

  it('has_pitch_graph_class', () => {
    const bufferRef = { current: [] };
    const { container } = render(
      <PitchGraph bufferRef={bufferRef} isCapturing={false} />,
    );
    expect(container.firstElementChild?.className).toBe('pitch-graph');
  });

  it('does_not_crash_with_data', () => {
    const now = Date.now();
    const bufferRef = {
      current: [
        { frequency_hz: 440, note_name: 'A4', cents_deviation: 0, clarity: 0.9, timestamp_ms: now - 2000 },
        { frequency_hz: 523, note_name: 'C5', cents_deviation: -2, clarity: 0.8, timestamp_ms: now - 1000 },
        { frequency_hz: 494, note_name: 'B4', cents_deviation: 15, clarity: 0.6, timestamp_ms: now },
      ],
    };
    const { container } = render(
      <PitchGraph bufferRef={bufferRef} isCapturing={true} />,
    );
    const canvas = container.querySelector('canvas');
    expect(canvas).toBeDefined();
  });
});
