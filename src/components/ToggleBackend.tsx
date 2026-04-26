import { Cloud, HardDrive } from 'lucide-react';
import type { Backend } from '../lib/tauri';

interface ToggleBackendProps {
  value: Backend;
  onChange: (value: Backend) => void;
}

export function ToggleBackend({ value, onChange }: ToggleBackendProps) {
  return (
    <div className="grid grid-cols-2 rounded-md border border-stone-200 bg-stone-100 p-1">
      <button
        type="button"
        className={`segmented-button ${value === 'groq' ? 'segmented-button-active' : ''}`}
        onClick={() => onChange('groq')}
      >
        <Cloud size={16} />
        Groq Cloud
      </button>
      <button
        type="button"
        className={`segmented-button ${value === 'local' ? 'segmented-button-active' : ''}`}
        onClick={() => onChange('local')}
      >
        <HardDrive size={16} />
        Local Whisper
      </button>
    </div>
  );
}
