import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import VolumeGraph from './VolumeGraph';

describe('VolumeGraph', () => {
  it('renders_canvas', () => {
    const bufferRef = { current: [] };
    const { container } = render(
      <VolumeGraph bufferRef={bufferRef} isCapturing={false} />,
    );
    const canvas = container.querySelector('canvas');
    expect(canvas).toBeDefined();
  });

  it('has_volume_graph_class', () => {
    const bufferRef = { current: [] };
    const { container } = render(
      <VolumeGraph bufferRef={bufferRef} isCapturing={false} />,
    );
    expect(container.firstElementChild?.className).toBe('volume-graph');
  });

  it('does_not_crash_with_data', () => {
    const now = Date.now();
    const bufferRef = {
      current: [
        { rms_level: 0.5, timestamp_ms: now - 2000 },
        { rms_level: 0.3, timestamp_ms: now - 1000 },
        { rms_level: 0.7, timestamp_ms: now },
      ],
    };
    const { container } = render(
      <VolumeGraph bufferRef={bufferRef} isCapturing={true} />,
    );
    const canvas = container.querySelector('canvas');
    expect(canvas).toBeDefined();
  });
});
