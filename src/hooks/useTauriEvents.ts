import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../store/useAppStore';
import type { ModelDownloadStatus, TranscriptionEntry } from '../lib/tauri';

interface CompletePayload {
  text: string;
  entry: TranscriptionEntry | null;
}

interface ErrorPayload {
  error: string;
}

export function useTauriEvents() {
  const setRecording = useAppStore((state) => state.setRecording);
  const setTranscribing = useAppStore((state) => state.setTranscribing);
  const setLastText = useAppStore((state) => state.setLastText);
  const setError = useAppStore((state) => state.setError);
  const setDownload = useAppStore((state) => state.setDownload);
  const addHistoryEntry = useAppStore((state) => state.addHistoryEntry);

  useEffect(() => {
    const unlisteners = [
      listen('recording-started', () => {
        setError(null);
        setRecording(true);
        setTranscribing(false);
      }),
      listen('recording-stopped', () => setRecording(false)),
      listen('transcription-started', () => setTranscribing(true)),
      listen<CompletePayload>('transcription-complete', (event) => {
        setTranscribing(false);
        setLastText(event.payload.text);
        if (event.payload.entry) {
          addHistoryEntry(event.payload.entry);
        }
      }),
      listen<ErrorPayload>('transcription-error', (event) => {
        setTranscribing(false);
        setError(event.payload.error);
      }),
      listen<ModelDownloadStatus>('model-download-progress', (event) => {
        setDownload(event.payload);
      }),
    ];

    return () => {
      unlisteners.forEach((unlisten) => {
        unlisten.then((fn) => fn()).catch(() => undefined);
      });
    };
  }, [addHistoryEntry, setDownload, setError, setLastText, setRecording, setTranscribing]);
}
