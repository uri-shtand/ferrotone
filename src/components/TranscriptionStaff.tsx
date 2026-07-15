import { useRef, useEffect } from 'react';
import type { NoteSegment } from '../types';

interface TranscriptionStaffProps {
  segmentsRef: React.MutableRefObject<NoteSegment[]>;
  isCapturing: boolean;
}

const WINDOW_MS = 60_000;
const MIDI_MIN = 48;
const MIDI_MAX = 84;

function centsDeviationColor(cents: number): string {
  const abs = Math.abs(cents);
  if (abs < 10) return '#4ade80';
  if (abs < 30) return '#fbbf24';
  return '#ef4444';
}

function diatonicIndex(noteIdx: number): number {
  const map = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];
  return map[(Math.round(noteIdx) % 12 + 12) % 12];
}

function midiToStaffY(midi: number, staffBottom: number, lineSpacing: number): number {
  const octave = Math.floor(Math.round(midi) / 12) - 1;
  const noteIdx = Math.round(midi) % 12;
  const diatonic = diatonicIndex(noteIdx);
  const pos = (octave - 4) * 7 + diatonic - 2;
  return staffBottom - pos * lineSpacing;
}

export default function TranscriptionStaff({ segmentsRef, isCapturing }: TranscriptionStaffProps) {
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

      const padL = 36;
      const padR = 8;
      const padTop = 20;
      const padB = 24;
      const plotW = w - padL - padR;
      const plotH = h - padTop - padB;

      if (plotW <= 0 || plotH <= 0) return;

      const staffTop = padTop + 4;
      const staffBottom = padTop + plotH - 8;
      const staffH = staffBottom - staffTop;
      const lineSpacing = staffH / 8;

      // Staff lines
      for (let pos = 0; pos <= 8; pos += 2) {
        const y = staffBottom - pos * lineSpacing;
        ctx.beginPath();
        ctx.moveTo(padL, y);
        ctx.lineTo(w - padR, y);
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.2)';
        ctx.lineWidth = 1;
        ctx.stroke();
      }

      // Octave labels on the left
      ctx.fillStyle = '#555';
      ctx.font = '10px Inter, Arial, sans-serif';
      ctx.textAlign = 'right';
      ctx.textBaseline = 'middle';
      for (let midi = MIDI_MIN; midi <= MIDI_MAX; midi += 12) {
        const y = midiToStaffY(midi, staffBottom, lineSpacing);
        ctx.fillText(`C${Math.floor(midi / 12) - 1}`, padL - 4, y);
      }

      if (!isCapturing || segmentsRef.current.length === 0) {
        ctx.fillStyle = '#555';
        ctx.font = '12px Inter, Arial, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText('Transcription', w / 2, padTop + plotH / 2);
        return;
      }

      const now = Date.now();
      const windowStart = now - WINDOW_MS;
      const windowEnd = now;

      // Draw note bars
      for (const seg of segmentsRef.current) {
        if (seg.end_timestamp_ms < windowStart || seg.start_timestamp_ms > windowEnd) continue;

        const x = padL + ((seg.start_timestamp_ms - windowStart) / WINDOW_MS) * plotW;
        const barW = Math.max(3, (seg.duration_ms / WINDOW_MS) * plotW);
        const barH = lineSpacing * 0.7;

        const y = midiToStaffY(seg.midi, staffBottom, lineSpacing) - barH / 2;

        const color = centsDeviationColor(seg.cents_deviation);

        // Draw rounded rectangle
        const r = 3;
        ctx.beginPath();
        ctx.moveTo(x + r, y);
        ctx.lineTo(x + barW - r, y);
        ctx.quadraticCurveTo(x + barW, y, x + barW, y + r);
        ctx.lineTo(x + barW, y + barH - r);
        ctx.quadraticCurveTo(x + barW, y + barH, x + barW - r, y + barH);
        ctx.lineTo(x + r, y + barH);
        ctx.quadraticCurveTo(x, y + barH, x, y + barH - r);
        ctx.lineTo(x, y + r);
        ctx.quadraticCurveTo(x, y, x + r, y);
        ctx.closePath();
        ctx.fillStyle = color;
        ctx.globalAlpha = 0.7;
        ctx.fill();
        ctx.strokeStyle = color;
        ctx.lineWidth = 1;
        ctx.globalAlpha = 1;
        ctx.stroke();

        // Note name label
        ctx.fillStyle = '#f6f6f6';
        ctx.font = '10px Inter, Arial, sans-serif';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(seg.note_name, x + barW / 2, y + barH / 2);
      }

      // Time axis
      ctx.fillStyle = '#555';
      ctx.font = '10px Inter, Arial, sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'top';
      const tickCount = 6;
      for (let i = 0; i <= tickCount; i++) {
        const frac = i / tickCount;
        const x = padL + frac * plotW;
        const secs = Math.round(((WINDOW_MS * (1 - frac)) / 1000));
        ctx.fillText(`-${secs}s`, x, staffBottom + 4);
      }
    };

    rafRef.current = requestAnimationFrame(draw);

    return () => {
      cancelAnimationFrame(rafRef.current);
      ro.disconnect();
    };
  }, [segmentsRef, isCapturing]);

  return (
    <div className="transcription-staff">
      <canvas ref={canvasRef} />
    </div>
  );
}
