import type { PitchFrame } from '../types';

interface PitchDisplayProps {
  isCapturing: boolean;
  latestFrame: PitchFrame | null;
  onStart: () => void;
  onStop: () => void;
}

function centsColor(cents: number, clarity: number): string {
  if (clarity < 0.5) return 'rgba(255, 255, 255, 0.4)';
  const abs = Math.abs(cents);
  if (abs < 5) return '#4ade80';
  if (abs < 25) return '#fbbf24';
  return '#ef4444';
}

function centsBarPosition(cents: number): number {
  const clamped = Math.max(-50, Math.min(50, cents));
  return ((clamped + 50) / 100) * 100;
}

export default function PitchDisplay({ isCapturing, latestFrame, onStart, onStop }: PitchDisplayProps) {
  const hasFrame = latestFrame !== null && isCapturing;
  const clarity = hasFrame ? latestFrame.clarity : 0;
  const color = hasFrame ? centsColor(latestFrame.cents_deviation, clarity) : '#888';
  const cents = hasFrame ? latestFrame.cents_deviation : 0;
  const noteName = hasFrame ? latestFrame.note_name : '--';
  const freq = hasFrame ? `${latestFrame.frequency_hz.toFixed(1)} Hz` : '--';
  const clarityPercent = `${(clarity * 100).toFixed(0)}%`;
  const clarityBarWidth = `${clarity * 100}%`;
  const centsStr = hasFrame ? `${cents >= 0 ? '+' : ''}${cents.toFixed(1)}c` : '--';
  const barPos = centsBarPosition(cents);

  return (
    <div className="pitch-display">
      <div className="note-section" style={{ color }}>
        <div className="note-name">{noteName}</div>
        <div className="note-cents">{centsStr}</div>
      </div>

      <div className="info-row">
        <span className="frequency">{freq}</span>
        <span className="clarity-meter">
          clarity:
          <span className="clarity-bar-track">
            <span className="clarity-bar-fill" style={{ width: clarityBarWidth }} />
          </span>
          {clarityPercent}
        </span>
      </div>

      <div className="cents-bar-container">
        <div className="cents-bar">
          <div className="cents-bar-tick" />
          <div className="cents-bar-indicator" style={{ left: `${barPos}%`, backgroundColor: color }} />
        </div>
        <div className="cents-bar-labels">
          <span>-50</span>
          <span>0</span>
          <span>+50</span>
        </div>
      </div>

      <div className="controls">
        {!isCapturing ? (
          <button className="btn btn-start" onClick={onStart}>
            &#9679; Start
          </button>
        ) : (
          <button className="btn btn-stop" onClick={onStop}>
            &#9632; Stop
          </button>
        )}
        <span className="status">
          {isCapturing ? 'Listening...' : latestFrame ? 'Stopped' : 'Idle'}
        </span>
      </div>
    </div>
  );
}
