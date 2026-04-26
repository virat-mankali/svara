import type React from 'react';
import { Keyboard } from 'lucide-react';

interface HotkeyRecorderProps {
  value: string;
  onChange: (value: string) => void;
}

export function HotkeyRecorder({ value, onChange }: HotkeyRecorderProps) {
  function capture(event: React.KeyboardEvent<HTMLInputElement>) {
    event.preventDefault();
    const parts: string[] = [];
    if (event.metaKey || event.ctrlKey) parts.push('CmdOrCtrl');
    if (event.altKey) parts.push('Alt');
    if (event.shiftKey) parts.push('Shift');

    const key = normalizeKey(event.key);
    if (key && !['Meta', 'Control', 'Alt', 'Shift'].includes(key)) {
      parts.push(key);
      onChange(parts.join('+'));
    }
  }

  return (
    <div className="relative">
      <Keyboard className="input-icon" size={16} />
      <input
        className="input pl-9"
        value={value}
        onKeyDown={capture}
        onChange={() => undefined}
        spellCheck={false}
      />
    </div>
  );
}

function normalizeKey(key: string) {
  if (key === ' ') return 'Space';
  if (key.length === 1) return key.toUpperCase();
  return key;
}
