# Svara

Svara is a privacy-first macOS tray app for voice-to-text transcription. It is built with Tauri 2, React, TypeScript, SQLite, Groq Whisper, and an optional local Whisper backend.

## Development

```bash
npm install
npm run tauri -- dev
```

## Production Build

```bash
npm run tauri -- build
```

Local Whisper support is implemented behind a Rust feature so the base app can build quickly:

```bash
cargo tauri build --features local-whisper
```

For the Metal-backed local Whisper build, install the extra native tooling:

```bash
brew install cmake
rustup component add rustfmt
```
