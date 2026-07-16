import { useRef, useEffect } from 'react';
import type { VolumeFrame } from '../types';

interface VolumeGraphProps {
  bufferRef: React.MutableRefObject<VolumeFrame[]>;
  isCapturing: boolean;
}

const WINDOW_MS = 60_000;
const DB_MIN = -60;
const DB_MAX = 0;

function rmsToDb(rms: number): number {
  return 20 * Math.log10(Math.max(rms, 1e-6));
}

export default function VolumeGraph({ bufferRef, isCapturing }: VolumeGraphProps) {
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

      // Background
      ctx.fillStyle = '#1a1a2e';
      ctx.fillRect(0, 0, w, h);

      if (!isCapturing || bufferRef.current.length === 0) {
        // Draw empty state label
        ctx.fillStyle = '#555';
        ctx.font = '12px Inter, Arial, sans-serif';
        ctx.textAlign = 'center';
        ctx.fillText('Volume', w / 2, 20);
        return;
      }

      const buffer = bufferRef.current;
      const now = buffer[buffer.length - 1].timestamp_ms;
      const cutoff = now - WINDOW_MS;

      // Filter points within window (use a local copy)
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

      // Build path
      ctx.beginPath();
      let first = true;

      for (let i = 0; i < points.length; i++) {
        const frac = (points[i].timestamp_ms - timeStart) / timeRange;
        const x = padL + frac * plotW;
        const db = rmsToDb(points[i].rms_level);
        const yNorm = 1 - (db - DB_MIN) / (DB_MAX - DB_MIN);
        const y = padTop + yNorm * plotH;

        if (first) {
          ctx.moveTo(x, y);
          first = false;
        } else {
          ctx.lineTo(x, y);
        }
      }

      // Close the fill shape (down to bottom, back to left, back up)
      const lastX = padL + plotW;
      ctx.lineTo(lastX, padTop + plotH);
      ctx.lineTo(padL, padTop + plotH);
      ctx.closePath();

      // Fill with gradient
      const grad = ctx.createLinearGradient(0, padTop, 0, padTop + plotH);
      grad.addColorStop(0, 'rgba(74, 222, 128, 0.7)');
      grad.addColorStop(0.4, 'rgba(74, 222, 128, 0.3)');
      grad.addColorStop(1, 'rgba(74, 222, 128, 0.05)');
      ctx.fillStyle = grad;
      ctx.fill();

      // Draw outline
      ctx.beginPath();
      for (let i = 0; i < points.length; i++) {
        const frac = (points[i].timestamp_ms - timeStart) / timeRange;
        const x = padL + frac * plotW;
        const db = rmsToDb(points[i].rms_level);
        const yNorm = 1 - (db - DB_MIN) / (DB_MAX - DB_MIN);
        const y = padTop + yNorm * plotH;

        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.strokeStyle = '#4ade80';
      ctx.lineWidth = 1.5;
      ctx.stroke();

      // Grid lines + labels
      ctx.fillStyle = '#666';
      ctx.font = '10px Inter, Arial, sans-serif';
      ctx.textAlign = 'right';
      ctx.textBaseline = 'middle';

      for (let db = DB_MIN; db <= DB_MAX; db += 20) {
        const yNorm = 1 - (db - DB_MIN) / (DB_MAX - DB_MIN);
        const y = padTop + yNorm * plotH;

        ctx.beginPath();
        ctx.moveTo(padL, y);
        ctx.lineTo(w - padR, y);
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.08)';
        ctx.lineWidth = 1;
        ctx.stroke();

        ctx.fillStyle = '#666';
        ctx.textAlign = 'right';
        ctx.fillText(`${db}`, padL - 4, y);
      }

      // Y-axis label
      ctx.fillStyle = '#555';
      ctx.font = '10px Inter, Arial, sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'top';
      ctx.fillText('dB', padL / 2, padTop);
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
