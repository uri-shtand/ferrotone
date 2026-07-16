import { Play, Square } from 'lucide-react';
import { Button } from './ui/button';
import type { PitchFrame } from '../types';

interface PitchDisplayProps {
  isCapturing: boolean;
  latestFrame: PitchFrame | null;
  onStart: () => void;
  onStop: () => void;
}

function centsColor(cents: number, clarity: number): string {
  if (clarity < 0.5) return 'hsl(0 0% 60% / 0.4)';
  const abs = Math.abs(cents);
  if (abs < 5) return 'hsl(142 71% 45%)';
  if (abs < 25) return 'hsl(48 96% 53%)';
  return 'hsl(0 84% 60%)';
}

function centsBarPosition(cents: number): number {
  const clamped = Math.max(-50, Math.min(50, cents));
  return ((clamped + 50) / 100) * 100;
}

export default function PitchDisplay({ isCapturing, latestFrame, onStart, onStop }: PitchDisplayProps) {
  const hasFrame = latestFrame !== null && isCapturing;
  const clarity = hasFrame ? latestFrame.clarity : 0;
  const voiced = hasFrame ? latestFrame.voiced : false;
  const color = hasFrame ? centsColor(latestFrame.cents_deviation, clarity) : 'hsl(0 0% 53%)';
  const cents = hasFrame ? latestFrame.cents_deviation : 0;
  const noteName = hasFrame ? latestFrame.note_name : '--';
  const freq = hasFrame ? `${latestFrame.frequency_hz.toFixed(1)} Hz` : '--';
  const clarityPercent = `${(clarity * 100).toFixed(0)}%`;
  const clarityBarWidth = `${clarity * 100}%`;
  const centsStr = hasFrame ? `${cents >= 0 ? '+' : ''}${cents.toFixed(1)}c` : '--';
  const barPos = centsBarPosition(cents);

  return (
    <div className="flex flex-col items-center gap-6">
      <div className="flex flex-col items-center gap-1" style={{ color }}>
        <div className="text-5xl font-bold tracking-wider leading-none">
          {noteName}
        </div>
        <div className="text-xl font-medium">
          {centsStr}
        </div>
      </div>

      <div className="flex justify-center gap-8 text-sm text-muted-foreground">
        <span>{freq}</span>
        <span className={voiced ? '' : 'text-muted-foreground/60'}>
          {voiced ? 'Voiced' : 'Unvoiced'}
        </span>
        <span className="flex items-center gap-2">
          clarity:
          <span className="inline-block w-[60px] h-2 bg-muted rounded-full overflow-hidden">
            <span className="block h-full bg-primary rounded-full transition-all duration-50" style={{ width: clarityBarWidth }} />
          </span>
          {clarityPercent}
        </span>
      </div>

      <div className="w-full max-w-xs">
        <div className="relative h-1 bg-muted rounded mb-1">
          <div className="absolute left-1/2 top-[-4px] w-0.5 h-3 bg-muted-foreground/60 -translate-x-1/2" />
          <div
            className="absolute top-[-8px] w-3 h-5 rounded -translate-x-1/2 transition-all duration-50"
            style={{ left: `${barPos}%`, backgroundColor: color }}
          />
        </div>
        <div className="flex justify-between text-xs text-muted-foreground/60">
          <span>-50</span>
          <span>0</span>
          <span>+50</span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        {!isCapturing ? (
          <Button onClick={onStart}>
            <Play className="mr-2 h-4 w-4 fill-current" />
            Start
          </Button>
        ) : (
          <Button variant="destructive" onClick={onStop}>
            <Square className="mr-2 h-4 w-4 fill-current" />
            Stop
          </Button>
        )}
        <span className="text-sm text-muted-foreground">
          {isCapturing ? 'Listening...' : latestFrame ? 'Stopped' : 'Idle'}
        </span>
      </div>
    </div>
  );
}
