import { useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { NoteEvent, NoteSegment } from '../types';

const WINDOW_MS = 60_000;

export function useNoteCapture(isCapturing: boolean) {
  const segmentsRef = useRef<NoteSegment[]>([]);
  const activeRef = useRef<NoteSegment | null>(null);

  useEffect(() => {
    if (!isCapturing) {
      segmentsRef.current = [];
      activeRef.current = null;
      return;
    }

    const unlisten = listen<NoteEvent>('note-event', (event) => {
      const ev = event.payload;
      const now = Date.now();

      if (ev.event_type === 'started') {
        const seg: NoteSegment = {
          note_name: ev.note_name,
          midi: ev.midi,
          cents_deviation: ev.cents_deviation,
          clarity: ev.clarity,
          duration_ms: 0,
          start_timestamp_ms: ev.timestamp_ms,
          end_timestamp_ms: 0,
        };
        activeRef.current = seg;
      } else {
        if (activeRef.current && activeRef.current.note_name === ev.note_name) {
          activeRef.current.duration_ms = ev.duration_ms;
          activeRef.current.end_timestamp_ms = ev.timestamp_ms;
          activeRef.current.cents_deviation = ev.cents_deviation;
          activeRef.current.clarity = ev.clarity;
          segmentsRef.current.push(activeRef.current);
          activeRef.current = null;
        }

        // Prune segments outside 60s window
        const cutoff = now - WINDOW_MS;
        segmentsRef.current = segmentsRef.current.filter(
          (s) => s.end_timestamp_ms >= cutoff,
        );
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [isCapturing]);

  return { segmentsRef };
}
