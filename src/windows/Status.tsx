import { type CSSProperties, useEffect, useMemo, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getAudioMeter } from '../lib/tauri';
import { useAppStore } from '../store/useAppStore';

const BAR_COUNT = 28;
const EMPTY_WAVE = Array.from({ length: BAR_COUNT }, () => 0);

export function Status() {
  const isRecording = useAppStore((state) => state.isRecording);
  const isTranscribing = useAppStore((state) => state.isTranscribing);
  const lastText = useAppStore((state) => state.lastText);
  const error = useAppStore((state) => state.error);
  const [wave, setWave] = useState(EMPTY_WAVE);

  useEffect(() => {
    if (!isRecording && !isTranscribing && (lastText || error)) {
      const timeout = window.setTimeout(() => {
        getCurrentWindow().hide();
      }, 1600);
      return () => window.clearTimeout(timeout);
    }
  }, [error, isRecording, isTranscribing, lastText]);

  useEffect(() => {
    if (!isRecording) {
      setWave(EMPTY_WAVE);
      return;
    }

    let cancelled = false;
    let inFlight = false;

    const updateWave = async () => {
      if (inFlight) return;
      inFlight = true;

      try {
        const meter = await getAudioMeter();
        if (cancelled) return;

        const liveLevel = meter.level < 0.045 ? 0 : Math.max(0, Math.min(0.96, meter.level * 1.2));
        setWave((previous) => {
          const next = previous.slice(1);
          return [...next, liveLevel];
        });
      } catch {
        if (!cancelled) {
          setWave((previous) => [...previous.slice(1), 0]);
        }
      } finally {
        inFlight = false;
      }
    };

    updateWave();
    const interval = window.setInterval(updateWave, 32);

    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, [isRecording]);

  const bars = useMemo(
    () =>
      wave.map((level, index) => {
        const centerBias = 1 - Math.abs(index - (BAR_COUNT - 1) / 2) / BAR_COUNT;
        const liquidVariation = 0.84 + Math.sin(index * 1.73) * 0.12;
        const displayLevel = Math.min(0.98, Math.pow(level, 0.55));
        const height =
          1 + Math.min(0.98, displayLevel * liquidVariation + centerBias * displayLevel * 0.1) * 24;
        return Math.round(height);
      }),
    [wave],
  );

  const state = isRecording ? 'recording' : isTranscribing ? 'transcribing' : 'inserting';

  return (
    <main className="status-shell" data-tauri-drag-region>
      <div className="flow-bar" data-state={state} data-tauri-drag-region>
        {state === 'recording' ? (
          <div className="flow-wave" aria-hidden="true">
            {bars.map((height, index) => (
              <span
                key={index}
                style={{ '--bar-height': `${height}px` } as CSSProperties}
              />
            ))}
          </div>
        ) : (
          <span className="flow-label">
            {state === 'transcribing' ? 'Transcribing' : 'Inserting'}
          </span>
        )}
        <span className="flow-state" aria-hidden="true" />
      </div>
    </main>
  );
}
