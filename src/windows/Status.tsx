import { useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Loader2 } from 'lucide-react';
import { useAppStore } from '../store/useAppStore';

export function Status() {
  const isRecording = useAppStore((state) => state.isRecording);
  const isTranscribing = useAppStore((state) => state.isTranscribing);
  const lastText = useAppStore((state) => state.lastText);

  useEffect(() => {
    if (!isRecording && !isTranscribing && lastText) {
      const timeout = window.setTimeout(() => {
        getCurrentWindow().hide();
      }, 2000);
      return () => window.clearTimeout(timeout);
    }
  }, [isRecording, isTranscribing, lastText]);

  const label = isRecording ? 'Recording' : isTranscribing ? 'Transcribing' : 'Done';

  return (
    <main className="status-shell" data-tauri-drag-region>
      <div className="status-pill" data-tauri-drag-region>
        {isRecording ? (
          <span className="recording-dot" />
        ) : isTranscribing ? (
          <Loader2 className="animate-spin text-leaf" size={18} />
        ) : (
          <span className="done-dot" />
        )}
        <span className="text-sm font-medium">{label}</span>
      </div>
    </main>
  );
}
