import { Clipboard, Trash2 } from 'lucide-react';
import type { TranscriptionEntry } from '../lib/tauri';

interface HistoryItemProps {
  entry: TranscriptionEntry;
  onDelete: (id: string) => void;
}

export function HistoryItem({ entry, onDelete }: HistoryItemProps) {
  const created = new Date(entry.createdAt);

  return (
    <article className="history-row">
      <div className="min-w-0 flex-1">
        <div className="mb-2 flex items-center gap-2">
          <span className={`badge ${entry.source === 'groq' ? 'badge-cloud' : 'badge-local'}`}>
            {entry.source === 'groq' ? 'Groq' : 'Local'}
          </span>
          <time className="text-xs text-stone-500">{created.toLocaleString()}</time>
        </div>
        <p className="whitespace-pre-wrap text-sm leading-6 text-ink">{entry.text}</p>
      </div>
      <div className="flex shrink-0 flex-col gap-1">
        <button
          className="icon-button"
          type="button"
          aria-label="Copy"
          onClick={() => navigator.clipboard.writeText(entry.text)}
        >
          <Clipboard size={16} />
        </button>
        <button
          className="icon-button danger"
          type="button"
          aria-label="Delete"
          onClick={() => onDelete(entry.id)}
        >
          <Trash2 size={16} />
        </button>
      </div>
    </article>
  );
}
