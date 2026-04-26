import { Eye, EyeOff, KeyRound, Save } from 'lucide-react';
import { useEffect, useState } from 'react';
import { getGroqApiKey, setGroqApiKey } from '../lib/tauri';

export function ApiKeyInput() {
  const [key, setKey] = useState('');
  const [visible, setVisible] = useState(false);
  const [saved, setSaved] = useState(false);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    getGroqApiKey()
      .then((savedKey) => {
        setLoaded(Boolean(savedKey));
      })
      .catch(() => undefined);
  }, []);

  async function save() {
    await setGroqApiKey(key);
    setSaved(true);
    setLoaded(true);
    window.setTimeout(() => setSaved(false), 1600);
  }

  async function loadSavedKey() {
    const savedKey = await getGroqApiKey();
    setKey(savedKey);
    setLoaded(Boolean(savedKey));
  }

  return (
    <div className="space-y-3">
      <label className="field-label" htmlFor="groq-key">
        Groq API Key
      </label>
      <div className="flex gap-2">
        <div className="relative flex-1">
          <KeyRound className="input-icon" size={16} />
          <input
            id="groq-key"
            className="input pl-9 pr-10"
            type={visible ? 'text' : 'password'}
            value={key}
            placeholder={loaded ? 'Saved locally' : 'Paste Groq key to update'}
            onChange={(event) => setKey(event.target.value)}
          />
          <button
            aria-label={visible ? 'Hide API key' : 'Show API key'}
            className="icon-button absolute right-1 top-1"
            type="button"
            onClick={() => setVisible((value) => !value)}
          >
            {visible ? <EyeOff size={16} /> : <Eye size={16} />}
          </button>
        </div>
        <button className="primary-button w-28" type="button" onClick={save}>
          <Save size={16} />
          {saved ? 'Saved' : 'Save'}
        </button>
      </div>
      <button className="text-xs font-semibold text-stone-500 hover:text-ink" type="button" onClick={loadSavedKey}>
        Load saved key
      </button>
    </div>
  );
}
