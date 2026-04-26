import {
  Bell,
  BookOpen,
  Clipboard,
  Home,
  Mic,
  MoreVertical,
  PanelLeft,
  Power,
  RefreshCw,
  RotateCcw,
  Scissors,
  Settings as SettingsIcon,
  Sparkles,
  Trash2,
  Type,
  Wand2,
} from 'lucide-react';
import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { ApiKeyInput } from '../components/ApiKeyInput';
import { HotkeyRecorder } from '../components/HotkeyRecorder';
import { ModelDownloader } from '../components/ModelDownloader';
import { ToggleBackend } from '../components/ToggleBackend';
import {
  AppSettings,
  TranscriptionEntry,
  deleteHistoryEntry,
  getHistory,
  getSettings,
  listAudioDevices,
  saveSettings,
  toggleRecording,
} from '../lib/tauri';
import { useAppStore } from '../store/useAppStore';

type GroupedHistory = Array<[string, TranscriptionEntry[]]>;

export function Settings() {
  const storeSettings = useAppStore((state) => state.settings);
  const updateSettings = useAppStore((state) => state.updateSettings);
  const history = useAppStore((state) => state.history);
  const setHistory = useAppStore((state) => state.setHistory);
  const isRecording = useAppStore((state) => state.isRecording);
  const isTranscribing = useAppStore((state) => state.isTranscribing);
  const error = useAppStore((state) => state.error);
  const [settings, setSettings] = useState<AppSettings>(storeSettings);
  const [devices, setDevices] = useState<string[]>([]);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getSettings().then((next) => {
      setSettings(next);
      updateSettings(next);
    });
    listAudioDevices().then(setDevices).catch(() => setDevices([]));
    refreshHistory();
  }, [updateSettings]);

  useEffect(() => setSettings(storeSettings), [storeSettings]);

  async function refreshHistory() {
    setHistory(await getHistory());
  }

  async function persist() {
    await saveSettings(settings);
    updateSettings(settings);
    setSaved(true);
    window.setTimeout(() => setSaved(false), 1600);
  }

  async function removeEntry(id: string) {
    await deleteHistoryEntry(id);
    await refreshHistory();
  }

  const grouped = useMemo(() => groupHistory(history), [history]);
  const totalWords = useMemo(
    () => history.reduce((sum, entry) => sum + countWords(entry.text), 0),
    [history],
  );
  const todayCount = useMemo(() => {
    const today = new Date().toDateString();
    return history.filter((entry) => new Date(entry.createdAt).toDateString() === today).length;
  }, [history]);
  const latestWords = history[0] ? countWords(history[0].text) : 0;

  return (
    <main className="app-shell bg-[#f7f5ef] text-[#262522]">
      <aside className="sidebar">
        <div className="window-dots" aria-hidden="true">
          <span className="bg-[#ef6a5d]" />
          <span className="bg-[#f4bf4f]" />
          <span className="bg-[#62c554]" />
        </div>

        <div className="brand-row">
          <div className="brand-mark">
            <span />
            <span />
            <span />
            <span />
          </div>
          <strong>Svara</strong>
          <span className="plan-badge">Local</span>
        </div>

        <nav className="nav-list" aria-label="Svara">
          <SidebarItem icon={<Home size={18} />} label="Home" active />
          <SidebarItem icon={<BookOpen size={18} />} label="Dictionary" />
          <SidebarItem icon={<Scissors size={18} />} label="Snippets" />
          <SidebarItem icon={<Type size={18} />} label="Style" />
          <SidebarItem icon={<Wand2 size={18} />} label="Transforms" />
          <SidebarItem icon={<Clipboard size={18} />} label="Scratchpad" />
        </nav>

        <div className="sidebar-spacer" />

        <div className="sidebar-card">
          <div className="flex items-center justify-between">
            <strong>Private by default</strong>
            <span className="text-stone-400">-</span>
          </div>
          <p>Groq for speed, local Whisper when you want everything on-device.</p>
        </div>

        <nav className="nav-list nav-list-bottom" aria-label="Settings">
          <SidebarItem icon={<SettingsIcon size={18} />} label="Settings" />
          <SidebarItem icon={<Sparkles size={18} />} label="Help" />
        </nav>
      </aside>

      <section className="workspace-panel">
        <header className="topbar">
          <button className="icon-button" type="button" aria-label="Toggle sidebar">
            <PanelLeft size={18} />
          </button>
          <div className="topbar-actions">
            <button className="icon-button" type="button" aria-label="Notifications">
              <Bell size={18} />
            </button>
            <button className="record-button" type="button" onClick={toggleRecording}>
              {isRecording ? <Power size={18} /> : <Mic size={18} />}
              {isRecording ? 'Stop' : isTranscribing ? 'Working' : 'Record'}
            </button>
          </div>
        </header>

        <div className="home-grid">
          <section className="content-column">
            <div className="hero-row">
              <div>
                <h1>Welcome back, snazzl</h1>
                <p>Svara is ready for your next dictation.</p>
              </div>
            </div>

            <div className="promo-banner">
              <div>
                <h2>Make Svara sound like you</h2>
                <p>Choose your backend, save your hotkey, and keep every transcript close.</p>
              </div>
              <button className="light-button" type="button" onClick={toggleRecording}>
                Start now
              </button>
            </div>

            {error && <p className="error-banner">{error}</p>}

            <div className="history-toolbar">
              <h2>Transcripts</h2>
              <button className="secondary-button" type="button" onClick={refreshHistory}>
                <RefreshCw size={16} />
                Refresh
              </button>
            </div>

            <TranscriptTimeline grouped={grouped} onDelete={removeEntry} />
          </section>

          <aside className="right-rail">
            <section className="stats-card">
              <Stat value={compactNumber(totalWords)} label="total words" />
              <Stat value={String(latestWords)} label="latest words" />
              <Stat value={String(todayCount)} label="today" />
            </section>

            <section className="settings-card">
              <h2>Voice</h2>
              <ToggleBackend
                value={settings.backend}
                onChange={(backend) => setSettings((current) => ({ ...current, backend }))}
              />
              <ApiKeyInput />
              <ModelDownloader path={settings.local_model_path} />
            </section>

            <section className="settings-card">
              <h2>Controls</h2>
              <div>
                <label className="field-label" htmlFor="hotkey">
                  Global Hotkey
                </label>
                <div className="mt-2">
                  <HotkeyRecorder
                    value={settings.hotkey}
                    onChange={(hotkey) => setSettings((current) => ({ ...current, hotkey }))}
                  />
                </div>
              </div>

              <div>
                <label className="field-label" htmlFor="audio-device">
                  Audio Input
                </label>
                <select
                  id="audio-device"
                  className="input mt-2"
                  value={settings.audio_device ?? ''}
                  onChange={(event) =>
                    setSettings((current) => ({
                      ...current,
                      audio_device: event.target.value || null,
                    }))
                  }
                >
                  <option value="">System default</option>
                  {devices.map((device) => (
                    <option key={device} value={device}>
                      {device}
                    </option>
                  ))}
                </select>
              </div>

              <div className="flex items-center justify-between gap-4">
                <div>
                  <label className="field-label">Launch at Login</label>
                  <p className="mt-1 text-sm text-stone-600">Start Svara with macOS.</p>
                </div>
                <label className="switch">
                  <input
                    type="checkbox"
                    checked={settings.autostart}
                    onChange={(event) =>
                      setSettings((current) => ({ ...current, autostart: event.target.checked }))
                    }
                  />
                  <span />
                </label>
              </div>

              <div className="flex justify-end gap-2">
                <button
                  className="secondary-button"
                  type="button"
                  onClick={() => getSettings().then(setSettings)}
                >
                  <RotateCcw size={16} />
                  Revert
                </button>
                <button className="primary-button w-28" type="button" onClick={persist}>
                  {saved ? 'Saved' : 'Save'}
                </button>
              </div>
            </section>
          </aside>
        </div>
      </section>
    </main>
  );
}

function SidebarItem({
  icon,
  label,
  active = false,
}: {
  icon: ReactNode;
  label: string;
  active?: boolean;
}) {
  return (
    <button className={`sidebar-item ${active ? 'sidebar-item-active' : ''}`} type="button">
      {icon}
      {label}
    </button>
  );
}

function Stat({ value, label }: { value: string; label: string }) {
  return (
    <div className="stat-row">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}

function TranscriptTimeline({
  grouped,
  onDelete,
}: {
  grouped: GroupedHistory;
  onDelete: (id: string) => void;
}) {
  if (grouped.length === 0) {
    return <div className="empty-history">No transcripts yet.</div>;
  }

  return (
    <div className="timeline">
      {grouped.map(([date, entries]) => (
        <section key={date} className="timeline-group">
          <h3>{date}</h3>
          <div className="transcript-stack">
            {entries.map((entry) => (
              <TranscriptRow key={entry.id} entry={entry} onDelete={onDelete} />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function TranscriptRow({
  entry,
  onDelete,
}: {
  entry: TranscriptionEntry;
  onDelete: (id: string) => void;
}) {
  const created = new Date(entry.createdAt);

  return (
    <article className="transcript-row">
      <time>{created.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</time>
      <p>{entry.text}</p>
      <span className={`badge ${entry.source === 'groq' ? 'badge-cloud' : 'badge-local'}`}>
        {entry.source === 'groq' ? 'Groq' : 'Local'}
      </span>
      <div className="row-actions">
        <button
          className="icon-button"
          type="button"
          aria-label="Copy transcript"
          onClick={() => navigator.clipboard.writeText(entry.text)}
        >
          <Clipboard size={16} />
        </button>
        <button
          className="icon-button danger"
          type="button"
          aria-label="Delete transcript"
          onClick={() => onDelete(entry.id)}
        >
          <Trash2 size={16} />
        </button>
        <button className="icon-button" type="button" aria-label="More actions">
          <MoreVertical size={16} />
        </button>
      </div>
    </article>
  );
}

function groupHistory(history: TranscriptionEntry[]): GroupedHistory {
  const groups = new Map<string, TranscriptionEntry[]>();

  for (const entry of history) {
    const label = formatDateLabel(new Date(entry.createdAt));
    groups.set(label, [...(groups.get(label) ?? []), entry]);
  }

  return Array.from(groups.entries());
}

function formatDateLabel(date: Date) {
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);

  if (date.toDateString() === today.toDateString()) return 'Today';
  if (date.toDateString() === yesterday.toDateString()) return 'Yesterday';

  return date
    .toLocaleDateString([], { month: 'long', day: 'numeric', year: 'numeric' })
    .toUpperCase();
}

function countWords(text: string) {
  return text.trim().split(/\s+/).filter(Boolean).length;
}

function compactNumber(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return String(value);
}
