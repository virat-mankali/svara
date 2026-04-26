# Svara — Voice-to-Text Desktop App (Wispr Flow Alternative)

> A free, privacy-first voice transcription app built with Tauri. Supports both Groq Cloud API and local Whisper inference. Runs as a system tray app with a global hotkey.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Feature List](#2-feature-list)
3. [Tech Stack](#3-tech-stack)
4. [Architecture](#4-architecture)
5. [File & Folder Structure](#5-file--folder-structure)
6. [Backend Implementation (Rust)](#6-backend-implementation-rust)
7. [Frontend Implementation (React + TypeScript)](#7-frontend-implementation-react--typescript)
8. [History Storage Design](#8-history-storage-design)
9. [Configuration & Settings Schema](#9-configuration--settings-schema)
10. [Build & Setup Instructions](#10-build--setup-instructions)
11. [macOS Permissions & Entitlements](#11-macos-permissions--entitlements)
12. [Environment Variables & Secrets](#12-environment-variables--secrets)
13. [Known Limitations & Future Improvements](#13-known-limitations--future-improvements)

---

## 1. Project Overview

**Svara** (Sanskrit for *voice/tone*) is a free, open-source alternative to Wispr Flow. It runs as a macOS system tray application and lets you transcribe speech to text using a global hotkey. Transcribed text is automatically injected into the currently focused window (paste behavior).

It supports two transcription backends:

| Mode | Description |
|---|---|
| **Groq Cloud** | Fast cloud inference via Groq's API using `whisper-large-v3-turbo`. Requires internet + API key. |
| **Local Whisper** | On-device inference using `whisper.cpp` via `whisper-rs`. No internet needed. Uses Metal on M2. |

The last 100 transcription entries are stored in a local SQLite database. When the 101st entry is created, the oldest entry is automatically deleted (sliding window).

---

## 2. Feature List

### Core
- [x] Global hotkey to start/stop recording (configurable, default: `Cmd+Shift+Space`)
- [x] Records microphone audio while hotkey is held or toggled
- [x] Transcribes audio via selected backend (Groq or Local)
- [x] Injects transcribed text into the currently focused application (simulates paste)
- [x] System tray icon with status indicators (idle, recording, transcribing)

### Settings
- [x] Toggle between Groq Cloud and Local Whisper
- [x] Groq API key input (stored securely in macOS Keychain via Tauri's `stronghold` or `keyring` crate)
- [x] Local model path configuration (auto-downloads `ggml-large-v3-turbo.bin` on first run)
- [x] Configurable global hotkey
- [x] Configurable audio input device

### History
- [x] Stores last 100 transcriptions in SQLite
- [x] History panel showing text, timestamp, and source (Groq / Local)
- [x] Copy any history entry to clipboard
- [x] Delete individual history entry
- [x] Clear all history

### UI
- [x] Floating mini status window (shows recording/transcribing state)
- [x] Settings window
- [x] History window
- [x] System tray right-click menu

---

## 3. Tech Stack

### Application Framework
- **Tauri 2.x** — Rust backend + WebView frontend

### Frontend
- **React 18** with **TypeScript**
- **Vite** (dev server + build)
- **Tailwind CSS** (styling)
- **Zustand** (global state management)
- **shadcn/ui** (UI components)
- **Lucide React** (icons)

### Backend (Rust)
| Crate | Purpose |
|---|---|
| `tauri` | App framework, IPC, window management |
| `tauri-plugin-global-shortcut` | Global hotkey registration |
| `tauri-plugin-autostart` | Launch on login |
| `tauri-plugin-notification` | OS notifications |
| `cpal` | Cross-platform audio capture |
| `hound` | WAV file encoding |
| `whisper-rs` | Local Whisper inference (whisper.cpp bindings with Metal) |
| `reqwest` | HTTP client for Groq API |
| `rusqlite` | SQLite storage for history |
| `serde` / `serde_json` | Serialization |
| `keyring` | Secure API key storage in macOS Keychain |
| `enigo` | Simulating keyboard input (paste injection) |
| `tokio` | Async runtime |
| `uuid` | Generating unique IDs for history entries |
| `chrono` | Timestamps |

---

## 4. Architecture

```
┌──────────────────────────────────────────────────────┐
│                     Tauri App                        │
│                                                      │
│  ┌──────────────┐        ┌────────────────────────┐  │
│  │   Frontend   │◄──────►│   Rust Backend         │  │
│  │  (React/TS)  │  IPC   │                        │  │
│  │              │        │  ┌──────────────────┐  │  │
│  │  - Settings  │        │  │  Audio Capture   │  │  │
│  │  - History   │        │  │  (cpal)          │  │  │
│  │  - Status    │        │  └────────┬─────────┘  │  │
│  └──────────────┘        │           │             │  │
│                          │           ▼             │  │
│  ┌──────────────┐        │  ┌──────────────────┐  │  │
│  │  System Tray │        │  │  Transcription   │  │  │
│  │  (Tauri)     │        │  │  Router          │  │  │
│  └──────────────┘        │  └──┬───────────────┘  │  │
│                          │     │         │         │  │
│  ┌──────────────┐        │     ▼         ▼         │  │
│  │  Global      │        │  ┌──────┐  ┌────────┐  │  │
│  │  Hotkey      │        │  │Groq  │  │Whisper │  │  │
│  │  Listener    │        │  │API   │  │Local   │  │  │
│  └──────────────┘        │  └──────┘  │(Metal) │  │  │
│                          │            └────────┘  │  │
│                          │  ┌──────────────────┐  │  │
│                          │  │  SQLite History  │  │  │
│                          │  │  (rusqlite)      │  │  │
│                          │  └──────────────────┘  │  │
│                          │  ┌──────────────────┐  │  │
│                          │  │  Text Injection  │  │  │
│                          │  │  (enigo + paste) │  │  │
│                          │  └──────────────────┘  │  │
│                          └────────────────────────┘  │
└──────────────────────────────────────────────────────┘
```

### Flow

```
User presses hotkey
        │
        ▼
Start recording audio via cpal
        │
User releases hotkey (or presses again to stop)
        │
        ▼
Encode raw PCM to WAV using hound
        │
        ▼
Route to selected backend:
  ├── Groq Cloud: POST /audio/transcriptions → get text
  └── Local Whisper: whisper-rs run inference → get text
        │
        ▼
Save to SQLite (delete oldest if count > 100)
        │
        ▼
Inject text via enigo (Cmd+V simulation after clipboard write)
        │
        ▼
Emit IPC event to frontend → update UI status
```

---

## 5. File & Folder Structure

```
svara/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   │   ├── icon.png
│   │   ├── icon.icns              # macOS
│   │   └── tray-icon.png          # System tray (22x22 or 16x16)
│   └── src/
│       ├── main.rs                # App entry, tray setup, command registration
│       ├── audio.rs               # Audio capture using cpal
│       ├── transcribe/
│       │   ├── mod.rs             # TranscriptionRouter — picks Groq or Local
│       │   ├── groq.rs            # Groq API integration
│       │   └── local.rs           # whisper-rs local inference
│       ├── history.rs             # SQLite CRUD for transcription history
│       ├── config.rs              # App config read/write (JSON file)
│       ├── keychain.rs            # Groq API key secure storage
│       ├── inject.rs              # Text injection via enigo
│       └── commands.rs            # All Tauri #[tauri::command] functions
│
├── src/
│   ├── main.tsx                   # React entry
│   ├── App.tsx                    # Router — renders correct window
│   ├── windows/
│   │   ├── Settings.tsx           # Settings panel
│   │   ├── History.tsx            # History list
│   │   └── Status.tsx             # Floating recording status indicator
│   ├── components/
│   │   ├── ToggleBackend.tsx      # Groq / Local toggle switch
│   │   ├── ApiKeyInput.tsx        # Groq API key input
│   │   ├── HistoryItem.tsx        # Single history row
│   │   ├── HotkeyRecorder.tsx     # Hotkey capture input
│   │   └── ModelDownloader.tsx    # Progress UI for model download
│   ├── store/
│   │   └── useAppStore.ts         # Zustand store (backend toggle, status, etc.)
│   ├── hooks/
│   │   └── useTauriEvents.ts      # Listen to backend IPC events
│   ├── lib/
│   │   └── tauri.ts               # Typed wrappers around invoke()
│   └── styles/
│       └── globals.css
│
├── package.json
├── vite.config.ts
├── tailwind.config.ts
├── tsconfig.json
├── .env.example
└── README.md
```

---

## 6. Backend Implementation (Rust)

### 6.1 `main.rs`

Responsibilities:
- Initialize Tauri app with plugins
- Create system tray with menu items: Open Settings, Open History, Quit
- Register global hotkey plugin
- Initialize SQLite DB on startup
- Register all Tauri commands

```rust
// Pseudocode outline
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::init())
        .plugin(tauri_plugin_autostart::init())
        .plugin(tauri_plugin_notification::init())
        .system_tray(build_tray())
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::get_history,
            commands::delete_history_entry,
            commands::clear_history,
            commands::save_settings,
            commands::get_settings,
            commands::set_groq_api_key,
            commands::get_groq_api_key,
            commands::download_local_model,
            commands::get_model_download_status,
        ])
        .setup(|app| {
            history::init_db(app)?;
            hotkey::register_default(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running svara");
}
```

### 6.2 `audio.rs`

- Use `cpal` to enumerate input devices
- Capture raw PCM f32 samples into a `Vec<f32>` buffer
- Expose `start_recording()` and `stop_recording() -> Vec<f32>` 
- After stop, convert PCM buffer to a WAV file using `hound` and write to a temp path (`$TMPDIR/svara_recording.wav`)
- The WAV spec must match Whisper's requirement: **16kHz, mono, 16-bit PCM**

```rust
// Audio capture spec
let config = cpal::StreamConfig {
    channels: 1,
    sample_rate: cpal::SampleRate(16000),
    buffer_size: cpal::BufferSize::Default,
};
```

If the device doesn't support 16kHz natively, resample using `rubato` crate before encoding.

### 6.3 `transcribe/groq.rs`

- Read API key from Keychain via `keychain.rs`
- POST multipart form to `https://api.groq.com/openai/v1/audio/transcriptions`
- Include fields: `file` (WAV bytes), `model: "whisper-large-v3-turbo"`, `response_format: "json"`
- Parse response JSON: `{ "text": "..." }`
- Return `Result<String, TranscribeError>`

```rust
pub async fn transcribe_groq(wav_path: &str, api_key: &str) -> Result<String> {
    let file_bytes = std::fs::read(wav_path)?;
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(file_bytes)
            .file_name("audio.wav")
            .mime_str("audio/wav")?)
        .text("model", "whisper-large-v3-turbo")
        .text("response_format", "json");

    let client = reqwest::Client::new();
    let res = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    let body: serde_json::Value = res.json().await?;
    Ok(body["text"].as_str().unwrap_or("").to_string())
}
```

### 6.4 `transcribe/local.rs`

- Use `whisper-rs` crate which wraps `whisper.cpp`
- `whisper.cpp` automatically uses Metal on Apple Silicon when compiled with `WHISPER_METAL=1`
- Load model from config path (default: `~/Library/Application Support/svara/models/ggml-large-v3-turbo.bin`)
- Keep the `WhisperContext` alive across calls (load once, reuse)
- Model download: fetch from Hugging Face — `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin` — with progress reporting via Tauri events

```rust
pub fn transcribe_local(wav_path: &str, model_path: &str) -> Result<String> {
    let ctx = WhisperContext::new_with_params(
        model_path,
        WhisperContextParameters::default()
    )?;

    let mut state = ctx.create_state()?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_progress(false);
    params.set_print_realtime(false);

    // Read WAV and convert to f32 samples
    let samples = read_wav_as_f32(wav_path)?;

    state.full(params, &samples)?;

    let num_segments = state.full_n_segments()?;
    let mut text = String::new();
    for i in 0..num_segments {
        text.push_str(state.full_get_segment_text(i)?.trim());
        text.push(' ');
    }

    Ok(text.trim().to_string())
}
```

> **Metal note**: To enable Metal in whisper-rs, set the feature flag in Cargo.toml:
> ```toml
> whisper-rs = { version = "0.11", features = ["metal"] }
> ```
> This will compile whisper.cpp with Metal support and use the M2 GPU automatically.

### 6.5 `history.rs`

SQLite schema:

```sql
CREATE TABLE IF NOT EXISTS transcriptions (
    id         TEXT PRIMARY KEY,
    text       TEXT NOT NULL,
    source     TEXT NOT NULL,   -- 'groq' | 'local'
    created_at TEXT NOT NULL    -- ISO 8601 timestamp
);
```

Functions to implement:

```rust
pub fn init_db(app: &tauri::App) -> Result<()>
pub fn insert_entry(db: &Connection, text: &str, source: &str) -> Result<()>
    // After insert, run: DELETE FROM transcriptions WHERE id NOT IN (
    //   SELECT id FROM transcriptions ORDER BY created_at DESC LIMIT 100
    // )
pub fn get_all_entries(db: &Connection) -> Result<Vec<TranscriptionEntry>>
pub fn delete_entry(db: &Connection, id: &str) -> Result<()>
pub fn clear_all(db: &Connection) -> Result<()>
```

DB file location: `~/Library/Application Support/svara/history.db`

### 6.6 `inject.rs`

Strategy: Write transcribed text to clipboard, then simulate `Cmd+V`.

```rust
use enigo::{Enigo, Key, KeyboardControllable};

pub fn inject_text(text: &str) {
    // 1. Write to clipboard using arboard crate
    let mut clipboard = arboard::Clipboard::new().unwrap();
    clipboard.set_text(text).unwrap();

    // 2. Small delay to ensure clipboard is set
    std::thread::sleep(std::time::Duration::from_millis(50));

    // 3. Simulate Cmd+V
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Meta);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Meta);
}
```

> **Note**: On macOS, Accessibility permissions are required for `enigo` to simulate key events. The app must prompt the user to enable Accessibility in System Settings > Privacy & Security > Accessibility.

### 6.7 `keychain.rs`

```rust
use keyring::Entry;

const SERVICE: &str = "svara";
const GROQ_KEY: &str = "groq_api_key";

pub fn save_api_key(key: &str) -> Result<()> {
    Entry::new(SERVICE, GROQ_KEY)?.set_password(key)?;
    Ok(())
}

pub fn get_api_key() -> Result<String> {
    Ok(Entry::new(SERVICE, GROQ_KEY)?.get_password()?)
}
```

### 6.8 `config.rs`

App config stored as JSON at `~/Library/Application Support/svara/config.json`.

```rust
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub backend: Backend,           // "groq" | "local"
    pub hotkey: String,             // e.g. "CmdOrCtrl+Shift+Space"
    pub audio_device: Option<String>,
    pub local_model_path: Option<String>,
    pub autostart: bool,
}

#[derive(Serialize, Deserialize)]
pub enum Backend {
    Groq,
    Local,
}
```

---

## 7. Frontend Implementation (React + TypeScript)

### 7.1 Window Types

Tauri supports multiple windows. Configure in `tauri.conf.json`:

- **`main`** — Hidden by default. Used as the logical app window.
- **`settings`** — Settings panel (600x500, not resizable)
- **`history`** — History list (400x600)
- **`status`** — Floating status indicator, always on top, transparent background, no frame (120x50)

> Open windows from system tray click events in Rust, or from frontend via `WebviewWindow.getByLabel(...)`.

### 7.2 Zustand Store (`useAppStore.ts`)

```typescript
interface AppStore {
  backend: 'groq' | 'local';
  isRecording: boolean;
  isTranscribing: boolean;
  lastText: string;
  history: TranscriptionEntry[];
  settings: AppSettings;

  setBackend: (b: 'groq' | 'local') => void;
  setRecording: (v: boolean) => void;
  setTranscribing: (v: boolean) => void;
  setHistory: (h: TranscriptionEntry[]) => void;
  updateSettings: (s: Partial<AppSettings>) => void;
}
```

### 7.3 IPC Typed Wrappers (`lib/tauri.ts`)

```typescript
import { invoke } from '@tauri-apps/api/core';

export const startRecording = () => invoke('start_recording');
export const stopRecording = () => invoke('stop_recording');
export const getHistory = (): Promise<TranscriptionEntry[]> => invoke('get_history');
export const deleteHistoryEntry = (id: string) => invoke('delete_history_entry', { id });
export const clearHistory = () => invoke('clear_history');
export const saveSettings = (settings: AppSettings) => invoke('save_settings', { settings });
export const getSettings = (): Promise<AppSettings> => invoke('get_settings');
export const setGroqApiKey = (key: string) => invoke('set_groq_api_key', { key });
export const getGroqApiKey = (): Promise<string> => invoke('get_groq_api_key');
```

### 7.4 Event Listeners (`hooks/useTauriEvents.ts`)

```typescript
import { listen } from '@tauri-apps/api/event';

// Events emitted from Rust backend:
// 'recording-started'
// 'recording-stopped'
// 'transcription-started'
// 'transcription-complete' -> payload: { text: string }
// 'transcription-error'    -> payload: { error: string }
// 'model-download-progress' -> payload: { percent: number }
```

### 7.5 Settings Panel (`windows/Settings.tsx`)

Sections:
1. **Transcription Backend** — Toggle switch: Groq Cloud / Local Whisper
2. **Groq API Key** — Password input, Save button, link to Groq console
3. **Local Model** — Show model status (downloaded/not), Download button with progress bar, model path display
4. **Hotkey** — Capture input (`HotkeyRecorder` component)
5. **Audio Device** — Dropdown with available input devices
6. **General** — Launch at login toggle

### 7.6 History Panel (`windows/History.tsx`)

- List of `HistoryItem` components (text, timestamp, source badge)
- "Copy" icon button per item
- "Delete" icon button per item
- "Clear All" button at top
- Auto-refresh on window focus

### 7.7 Status Indicator (`windows/Status.tsx`)

Frameless, always-on-top floating window. Shows:
- 🔴 Pulsing red dot + "Recording..." when recording
- 🔵 Spinner + "Transcribing..." when waiting for result
- Hides itself after injection is complete (2s auto-dismiss)

Position: bottom-right corner of screen (use Tauri's `Window::set_position`).

---

## 8. History Storage Design

### Sliding Window (100 entries max)

After every `INSERT`, run the cleanup query:

```sql
DELETE FROM transcriptions
WHERE id NOT IN (
    SELECT id FROM transcriptions
    ORDER BY created_at DESC
    LIMIT 100
);
```

This keeps only the most recent 100 entries and is safe to run after every insert.

### Data Model

```typescript
interface TranscriptionEntry {
  id: string;           // UUID v4
  text: string;
  source: 'groq' | 'local';
  createdAt: string;    // ISO 8601
}
```

---

## 9. Configuration & Settings Schema

### `config.json` (stored in App Support dir)

```json
{
  "backend": "groq",
  "hotkey": "CmdOrCtrl+Shift+Space",
  "audio_device": null,
  "local_model_path": null,
  "autostart": false
}
```

### Groq API Key

Stored in macOS Keychain, **never** written to disk in plaintext.

### Local Model

- Default download path: `~/Library/Application Support/svara/models/ggml-large-v3-turbo.bin`
- Model size: ~1.5 GB
- Source: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`
- Download via `reqwest` with streaming + progress events emitted to frontend

---

## 10. Build & Setup Instructions

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (v18+)
brew install node

# Install Tauri CLI
cargo install tauri-cli --version "^2.0"

# Required system deps for whisper.cpp Metal build
xcode-select --install
```

### Dev Setup

```bash
git clone https://github.com/virat-mankali/svara
cd svara

npm install

# Run in dev mode
cargo tauri dev
```

### Build for Production

```bash
cargo tauri build
```

Output: `src-tauri/target/release/bundle/macos/Svara.app`

### Cargo.toml Dependencies

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-autostart = "2"
tauri-plugin-notification = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
cpal = "0.15"
hound = "3.5"
rubato = "0.14"           # Audio resampling if device != 16kHz
whisper-rs = { version = "0.11", features = ["metal"] }
reqwest = { version = "0.12", features = ["multipart", "json", "stream"] }
rusqlite = { version = "0.31", features = ["bundled"] }
keyring = "2"
enigo = "0.2"
arboard = "3"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

### `tauri.conf.json` Key Settings

```json
{
  "app": {
    "withGlobalTauri": true
  },
  "bundle": {
    "identifier": "com.viratmankali.svara",
    "icon": ["icons/icon.icns"],
    "macOS": {
      "entitlements": "entitlements.plist",
      "signingIdentity": null
    }
  },
  "windows": [
    {
      "label": "main",
      "visible": false,
      "width": 1,
      "height": 1
    },
    {
      "label": "settings",
      "title": "Svara Settings",
      "width": 600,
      "height": 520,
      "resizable": false,
      "visible": false
    },
    {
      "label": "history",
      "title": "Transcription History",
      "width": 420,
      "height": 620,
      "resizable": true,
      "visible": false
    },
    {
      "label": "status",
      "title": "",
      "width": 160,
      "height": 44,
      "decorations": false,
      "transparent": true,
      "alwaysOnTop": true,
      "visible": false,
      "skipTaskbar": true
    }
  ]
}
```

---

## 11. macOS Permissions & Entitlements

### `entitlements.plist`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>com.apple.security.device.audio-input</key>
  <true/>
  <key>com.apple.security.automation.apple-events</key>
  <true/>
</dict>
</plist>
```

### `Info.plist` Keys (add to Tauri's generated plist)

```xml
<key>NSMicrophoneUsageDescription</key>
<string>Svara needs microphone access to record your voice for transcription.</string>
<key>NSAppleEventsUsageDescription</key>
<string>Svara needs Accessibility access to inject transcribed text into other apps.</string>
```

### Runtime Permission Prompts

On first launch, check and prompt for:
1. **Microphone** — Request via Tauri plugin or `AVCaptureDevice.requestAccess`
2. **Accessibility** — Check via `AXIsProcessTrusted()`. If false, open System Settings to the Accessibility pane and show an in-app prompt.

Implement these checks in `setup()` in `main.rs`.

---

## 12. Environment Variables & Secrets

| Variable | Where | Purpose |
|---|---|---|
| `GROQ_API_KEY` | macOS Keychain | Groq Cloud API key |
| None others | — | Everything else is in `config.json` |

`.env.example` (for dev only, do not commit real keys):

```env
# Development only - do not commit
GROQ_API_KEY=gsk_your_key_here
```

---

## 13. Known Limitations & Future Improvements

### Current Limitations

- **macOS only** — `enigo` paste injection and Metal are macOS-specific. Windows/Linux support would need separate implementations.
- **Accessibility required** — Text injection requires Accessibility permission. Without it, the app copies to clipboard only.
- **Model download is large** — `ggml-large-v3-turbo.bin` is ~1.5 GB. Consider offering `ggml-base.en.bin` (~75 MB) as a faster/lighter alternative in settings.
- **No real-time streaming** — Transcription starts only after recording stops. Streaming transcription would require a different architecture.

### Possible Future Features

- **Language detection / multi-language** — Expose Whisper's language param in Settings
- **Custom vocabulary / prompt** — Pass initial prompt to Whisper for domain-specific terms
- **Auto-punctuation mode** — Use Groq/OpenAI to post-process and add punctuation
- **Transcription streaming** — Stream partial results to the status window while processing
- **Windows support** — Use DirectX/CUDA instead of Metal for local inference
- **Export history** — Export past 100 entries to CSV or plain text

---

*Built by Virat Mankali — [github.com/virat-mankali](https://github.com/virat-mankali) · [@viratt_mankali](https://twitter.com/viratt_mankali)*
