import { useRef, useEffect } from 'react';
import type { PitchFrame } from '../types';

interface PitchGraphProps {
  bufferRef: React.MutableRefObject<PitchFrame[]>;
  isCapturing: boolean;
}

const WINDOW_MS = 60_000;
const MIDI_MIN = 36;
const MIDI_MAX = 84;

function hzToMidi(f: number): number {
  if (f <= 0) return MIDI_MIN;
  return 69 + 12 * Math.log2(f / 440);
}

function centsDeviationColor(cents: number): string {
  const abs = Math.abs(cents);
  if (abs < 10) return '#4ade80';
  if (abs < 30) return '#fbbf24';
  return '#ef4444';
}

function midiToNoteName(midi: number): string {
  const notes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];
  const octave = Math.floor(midi / 12) - 1;
  const noteIdx = Math.round(midi) % 12;
  return `${notes[noteIdx]}${octave}`;
}

export default function PitchGraph({ bufferRef, isCapturing }: PitchGraphProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const rafRef = useRef<number>(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resize = () => {
      const parent = canvas.parentElement;
      if (!parent) return;
      const dpr = window.devicePixelRatio || 1;
      const w = parent.clientWidth;
      const h = parent.clientHeight;
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      canvas.style.width = `${w}px`;
      canvas.style.height = `${h}px`;
      ctx.scale(dpr, dpr);
    };

    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(canvas.parentElement!);

    const draw = () => {
      rafRef.current = requestAnimationFrame(draw);

      const parent = canvas.parentElement;
      if (!parent) return;
      const w = parent.clientWidth;
      const h = parent.clientHeight;

      ctx.clearRect(0, 0, w, h);

      ctx.fillStyle = '#1a1a2e';
      ctx.fillRect(0, 0, w, h);

      if (!isCapturing || bufferRef.current.length === 0) {
        ctx.fillStyle = '#555';
        ctx.font = '12px Inter, Arial, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('Pitch', w / 2, 20);
        return;
      }

      const buffer = bufferRef.current;
      const now = buffer[buffer.length - 1].timestamp_ms;
      const cutoff = now - WINDOW_MS;

      const points = buffer.filter((f) => f.timestamp_ms >= cutoff);
      if (points.length < 2) return;

      const padL = 40;
      const padR = 8;
      const padTop = 24;
      const padB = 16;
      const plotW = w - padL - padR;
      const plotH = h - padTop - padB;

      if (plotW <= 0 || plotH <= 0) return;

      const timeStart = now - WINDOW_MS;
      const timeEnd = now;
      const timeRange = Math.max(timeEnd - timeStart, 1);

      // Grid lines at octave boundaries
      ctx.fillStyle = '#666';
      ctx.font = '10px Inter, Arial, sans-serif';
      ctx.textAlign = 'right';
      ctx.textBaseline = 'middle';

      for (let midi = MIDI_MIN; midi <= MIDI_MAX; midi += 12) {
        const yNorm = 1 - (midi - MIDI_MIN) / (MIDI_MAX - MIDI_MIN);
        const y = padTop + yNorm * plotH;

        ctx.beginPath();
        ctx.moveTo(padL, y);
        ctx.lineTo(w - padR, y);
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)';
        ctx.lineWidth = 1;
        ctx.stroke();

        ctx.fillStyle = '#666';
        ctx.fillText(midiToNoteName(midi), padL - 4, y);
      }

      // Y-axis label
      ctx.fillStyle = '#555';
      ctx.font = '10px Inter, Arial, sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'top';
      ctx.fillText('MIDI', padL / 2, padTop);

      // Build area fill path (using cents color at the last point)
      ctx.beginPath();
      let first = true;
      for (let i = 0; i < points.length; i++) {
        const frac = (points[i].timestamp_ms - timeStart) / timeRange;
        const x = padL + frac * plotW;
        const midiVal = hzToMidi(points[i].frequency_hz);
        const yNorm = 1 - (midiVal - MIDI_MIN) / (MIDI_MAX - MIDI_MIN);
        const y = padTop + Math.min(Math.max(yNorm * plotH, 0), plotH);

        if (first) {
          ctx.moveTo(x, y);
          first = false;
        } else {
          ctx.lineTo(x, y);
        }
      }

      const lastX = padL + plotW;
      ctx.lineTo(lastX, padTop + plotH);
      ctx.lineTo(padL, padTop + plotH);
      ctx.closePath();

      const grad = ctx.createLinearGradient(0, padTop, 0, padTop + plotH);
      const lastCents = points[points.length - 1].cents_deviation;
      const centsColor = centsDeviationColor(lastCents);
      grad.addColorStop(0, `${centsColor}b3`);
      grad.addColorStop(0.4, `${centsColor}4d`);
      grad.addColorStop(1, `${centsColor}0d`);
      ctx.fillStyle = grad;
      ctx.fill();

      // Draw stroke line with per-point cents coloring
      ctx.beginPath();
      for (let i = 0; i < points.length; i++) {
        const frac = (points[i].timestamp_ms - timeStart) / timeRange;
        const x = padL + frac * plotW;
        const midiVal = hzToMidi(points[i].frequency_hz);
        const yNorm = 1 - (midiVal - MIDI_MIN) / (MIDI_MAX - MIDI_MIN);
        const y = padTop + Math.min(Math.max(yNorm * plotH, 0), plotH);

        if (i === 0) {
          ctx.moveTo(x, y);
        } else {
          const prevX = padL + ((points[i - 1].timestamp_ms - timeStart) / timeRange) * plotW;
          const prevMidi = hzToMidi(points[i - 1].frequency_hz);
          const prevYNorm = 1 - (prevMidi - MIDI_MIN) / (MIDI_MAX - MIDI_MIN);
          const prevY = padTop + Math.min(Math.max(prevYNorm * plotH, 0), plotH);

          const midColor = centsDeviationColor(
            (points[i - 1].cents_deviation + points[i].cents_deviation) / 2,
          );

          ctx.strokeStyle = midColor;
          ctx.lineWidth = 1.5;

          ctx.beginPath();
          ctx.moveTo(prevX, prevY);
          ctx.lineTo(x, y);
          ctx.stroke();
        }
      }
    };

    rafRef.current = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(rafRef.current);
      ro.disconnect();
    };
  }, [bufferRef, isCapturing]);

  return (
    <div className="w-full h-[120px] mt-6 border-t border-border">
      <canvas ref={canvasRef} className="block w-full h-full" />
    </div>
  );
}
