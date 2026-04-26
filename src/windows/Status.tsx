import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAppStore } from '../store/useAppStore';

export function Status() {
  const isRecording = useAppStore((state) => state.isRecording);
  const isTranscribing = useAppStore((state) => state.isTranscribing);
  const lastText = useAppStore((state) => state.lastText);
  const error = useAppStore((state) => state.error);

  useEffect(() => {
    if (!isRecording && !isTranscribing && (lastText || error)) {
      const timeout = window.setTimeout(() => {
        getCurrentWindow().hide();
      }, 1600);
      return () => window.clearTimeout(timeout);
    }
  }, [error, isRecording, isTranscribing, lastText]);

  const state = isRecording ? 'recording' : isTranscribing ? 'transcribing' : 'inserting';

  return (
    <main className="status-shell" data-tauri-drag-region>
      <div className="flow-bar" data-state={state} data-tauri-drag-region>
        {state === 'recording' ? (
          <div className="flow-wave" aria-hidden="true">
            <span />
            <span />
            <span />
            <span />
            <span />
            <span />
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
