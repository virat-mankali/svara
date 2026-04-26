import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Mic } from 'lucide-react';
import { useAppStore } from '../store/useAppStore';

export function Status() {
  const isRecording = useAppStore((state) => state.isRecording);
  const isTranscribing = useAppStore((state) => state.isTranscribing);
  const lastText = useAppStore((state) => state.lastText);

  useEffect(() => {
    if (!isRecording && !isTranscribing && lastText) {
      const timeout = window.setTimeout(() => {
        getCurrentWindow().hide();
      }, 1600);
      return () => window.clearTimeout(timeout);
    }
  }, [isRecording, isTranscribing, lastText]);

  const title = isRecording ? 'Listening' : isTranscribing ? 'Polishing' : 'Inserted';
  const subtitle = isRecording ? 'Speak naturally' : isTranscribing ? 'Turning voice into text' : 'Ready';
  const stateClass = isRecording
    ? 'flow-state'
    : isTranscribing
      ? 'flow-state flow-state-working'
      : 'flow-state flow-state-idle';

  return (
    <main className="status-shell" data-tauri-drag-region>
      <div className="flow-bar" data-tauri-drag-region>
        <div className="flow-orb">
          {isRecording ? <span className="flow-orb-dot" /> : <Mic size={16} />}
        </div>
        <div className="flow-wave" aria-hidden="true">
          <span />
          <span />
          <span />
          <span />
          <span />
          <span />
        </div>
        <div className="flow-copy">
          <strong>{title}</strong>
          <span>{subtitle}</span>
        </div>
        <span className={stateClass} />
      </div>
    </main>
  );
}
