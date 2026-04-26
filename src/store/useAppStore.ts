import { create } from 'zustand';
import type { AppSettings, Backend, ModelDownloadStatus, TranscriptionEntry } from '../lib/tauri';

interface AppStore {
  backend: Backend;
  isRecording: boolean;
  isTranscribing: boolean;
  lastText: string;
  error: string | null;
  history: TranscriptionEntry[];
  download: ModelDownloadStatus;
  settings: AppSettings;
  setBackend: (backend: Backend) => void;
  setRecording: (value: boolean) => void;
  setTranscribing: (value: boolean) => void;
  setLastText: (text: string) => void;
  setError: (error: string | null) => void;
  setHistory: (history: TranscriptionEntry[]) => void;
  addHistoryEntry: (entry: TranscriptionEntry) => void;
  setDownload: (download: ModelDownloadStatus) => void;
  updateSettings: (settings: Partial<AppSettings>) => void;
}

const defaultSettings: AppSettings = {
  backend: 'groq',
  hotkey: 'CmdOrCtrl+Shift+Space',
  audio_device: null,
  local_model_path: null,
  autostart: false,
};

export const useAppStore = create<AppStore>((set) => ({
  backend: 'groq',
  isRecording: false,
  isTranscribing: false,
  lastText: '',
  error: null,
  history: [],
  download: { percent: 0, is_downloading: false, message: 'Not started' },
  settings: defaultSettings,
  setBackend: (backend) =>
    set((state) => ({
      backend,
      settings: { ...state.settings, backend },
    })),
  setRecording: (isRecording) => set({ isRecording }),
  setTranscribing: (isTranscribing) => set({ isTranscribing }),
  setLastText: (lastText) => set({ lastText }),
  setError: (error) => set({ error }),
  setHistory: (history) => set({ history }),
  addHistoryEntry: (entry) =>
    set((state) => ({
      history: [entry, ...state.history.filter((item) => item.id !== entry.id)].slice(0, 100),
    })),
  setDownload: (download) => set({ download }),
  updateSettings: (settings) =>
    set((state) => ({
      backend: settings.backend ?? state.backend,
      settings: { ...state.settings, ...settings },
    })),
}));
