import { RefreshCw, Trash2 } from 'lucide-react';
import { useEffect } from 'react';
import { HistoryItem } from '../components/HistoryItem';
import { clearHistory, deleteHistoryEntry, getHistory } from '../lib/tauri';
import { useAppStore } from '../store/useAppStore';

export function History() {
  const history = useAppStore((state) => state.history);
  const setHistory = useAppStore((state) => state.setHistory);

  async function refresh() {
    setHistory(await getHistory());
  }

  async function remove(id: string) {
    await deleteHistoryEntry(id);
    await refresh();
  }

  async function clear() {
    await clearHistory();
    await refresh();
  }

  useEffect(() => {
    refresh();
    window.addEventListener('focus', refresh);
    return () => window.removeEventListener('focus', refresh);
  }, []);

  return (
    <main className="min-h-screen bg-paper text-ink">
      <div className="mx-auto flex max-w-xl flex-col gap-4 px-5 py-5">
        <header className="flex items-center justify-between gap-3">
          <div>
            <h1 className="text-xl font-semibold">History</h1>
            <p className="text-sm text-stone-600">The most recent 100 transcriptions.</p>
          </div>
          <div className="flex gap-1">
            <button className="icon-button" type="button" aria-label="Refresh" onClick={refresh}>
              <RefreshCw size={16} />
            </button>
            <button className="icon-button danger" type="button" aria-label="Clear all" onClick={clear}>
              <Trash2 size={16} />
            </button>
          </div>
        </header>

        <section className="flex flex-col gap-3">
          {history.length === 0 ? (
            <div className="panel py-12 text-center text-sm text-stone-500">No transcriptions yet.</div>
          ) : (
            history.map((entry) => <HistoryItem key={entry.id} entry={entry} onDelete={remove} />)
          )}
        </section>
      </div>
    </main>
  );
}
