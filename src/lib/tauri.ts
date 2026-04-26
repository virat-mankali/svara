import { invoke } from '@tauri-apps/api/core';

export type Backend = 'groq' | 'local';

export interface AppSettings {
  backend: Backend;
  hotkey: string;
  audio_device: string | null;
  local_model_path: string | null;
  groq_api_key?: string | null;
  autostart: boolean;
}

export interface TranscriptionEntry {
  id: string;
  text: string;
  source: Backend;
  createdAt: string;
}

export interface ModelDownloadStatus {
  percent: number;
  is_downloading: boolean;
  message: string;
}

export const startRecording = () => invoke<void>('start_recording');
export const stopRecording = () => invoke<string>('stop_recording');
export const toggleRecording = () => invoke<void>('toggle_recording_command');
export const getHistory = () => invoke<TranscriptionEntry[]>('get_history');
export const deleteHistoryEntry = (id: string) => invoke<void>('delete_history_entry', { id });
export const clearHistory = () => invoke<void>('clear_history');
export const saveSettings = (settings: AppSettings) => invoke<void>('save_settings', { settings });
export const getSettings = () => invoke<AppSettings>('get_settings');
export const setGroqApiKey = (key: string) => invoke<void>('set_groq_api_key', { key });
export const getGroqApiKey = () => invoke<string>('get_groq_api_key');
export const downloadLocalModel = () => invoke<string>('download_local_model');
export const getModelDownloadStatus = () =>
  invoke<ModelDownloadStatus>('get_model_download_status');
export const listAudioDevices = () => invoke<string[]>('list_audio_devices');
export const openAccessibilitySettings = () => invoke<void>('open_accessibility_settings');
export const openMicrophoneSettings = () => invoke<void>('open_microphone_settings');
