import { CheckCircle2, Download, FolderOpen } from 'lucide-react';
import { useEffect } from 'react';
import { downloadLocalModel, getModelDownloadStatus } from '../lib/tauri';
import { useAppStore } from '../store/useAppStore';

interface ModelDownloaderProps {
  path: string | null;
}

export function ModelDownloader({ path }: ModelDownloaderProps) {
  const download = useAppStore((state) => state.download);
  const setDownload = useAppStore((state) => state.setDownload);
  const isDownloaded = download.percent >= 100 && download.message === 'Downloaded';

  useEffect(() => {
    getModelDownloadStatus().then(setDownload).catch(() => undefined);
  }, [path, setDownload]);

  async function startDownload() {
    await downloadLocalModel();
    const status = await getModelDownloadStatus();
    setDownload(status);
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between gap-4">
        <div className="min-w-0">
          <label className="field-label">Local Model</label>
          <p className="mt-1 flex items-center gap-2 truncate text-sm text-stone-600">
            <FolderOpen size={15} />
            <span className="truncate">{path ?? 'Default model path'}</span>
          </p>
        </div>
        <button
          className="secondary-button shrink-0"
          type="button"
          disabled={download.is_downloading || isDownloaded}
          onClick={startDownload}
        >
          {isDownloaded ? <CheckCircle2 size={16} /> : <Download size={16} />}
          {isDownloaded ? 'Downloaded' : 'Download'}
        </button>
      </div>
      <div className="h-2 overflow-hidden rounded-full bg-stone-200">
        <div
          className="h-full bg-leaf transition-all"
          style={{ width: `${Math.max(0, Math.min(100, download.percent))}%` }}
        />
      </div>
      <p className="text-xs text-stone-500">{download.message}</p>
    </div>
  );
}
