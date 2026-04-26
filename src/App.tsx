import { useEffect, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { History } from './windows/History';
import { Settings } from './windows/Settings';
import { Status } from './windows/Status';
import { useTauriEvents } from './hooks/useTauriEvents';
import { useAppStore } from './store/useAppStore';
import { getSettings } from './lib/tauri';

type WindowLabel = 'main' | 'settings' | 'history' | 'status';

export default function App() {
  const [label, setLabel] = useState<WindowLabel>('settings');
  const updateSettings = useAppStore((state) => state.updateSettings);

  useTauriEvents();

  useEffect(() => {
    setLabel(getCurrentWindow().label as WindowLabel);
    getSettings().then(updateSettings).catch(() => undefined);
  }, [updateSettings]);

  if (label === 'history') return <History />;
  if (label === 'status') return <Status />;
  return <Settings />;
}
