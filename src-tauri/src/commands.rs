use futures_util::StreamExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State};
use tauri_plugin_autostart::ManagerExt;

use crate::audio;
use crate::config::{self, AppConfig};
use crate::history::TranscriptionEntry;
use crate::{inject, reregister_hotkey, AppState, ModelDownloadStatus};

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin";

#[derive(Clone, Serialize)]
struct TranscriptionCompletePayload {
    text: String,
    entry: Option<TranscriptionEntry>,
}

#[derive(Clone, Serialize)]
struct ErrorPayload {
    error: String,
}

#[tauri::command]
pub async fn start_recording(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    start_recording_inner(&app, &state).await
}

#[tauri::command]
pub async fn stop_recording(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    stop_recording_inner(&app, &state).await
}

#[tauri::command]
pub async fn toggle_recording_command(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.recorder.is_recording() {
        stop_recording_inner(&app, &state).await.map(|_| ())
    } else {
        start_recording_inner(&app, &state).await
    }
}

pub async fn toggle_recording(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if state.recorder.is_recording() {
        stop_recording_inner(&app, &state).await.map(|_| ())
    } else {
        start_recording_inner(&app, &state).await
    }
}

async fn start_recording_inner(app: &AppHandle, state: &State<'_, AppState>) -> Result<(), String> {
    let device = state.config.lock().unwrap().audio_device.clone();
    state
        .recorder
        .start(device.as_deref())
        .map_err(|error| error.to_string())?;

    let _ = app.emit("recording-started", ());
    show_status(app);
    Ok(())
}

async fn stop_recording_inner(
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<String, String> {
    let wav_path = state
        .recorder
        .stop_to_wav()
        .map_err(|error| error.to_string())?;

    let _ = app.emit("recording-stopped", ());
    let _ = app.emit("transcription-started", ());
    show_status(app);

    let config = state.config.lock().unwrap().clone();
    let source = match config.backend {
        config::Backend::Groq => "groq",
        config::Backend::Local => "local",
    }
    .to_string();

    let text = match crate::transcribe::transcribe(&wav_path, &config).await {
        Ok(text) => text,
        Err(error) => {
            let message = error.to_string();
            let _ = app.emit(
                "transcription-error",
                ErrorPayload {
                    error: message.clone(),
                },
            );
            return Err(message);
        }
    };

    let entry = if text.trim().is_empty() {
        None
    } else {
        let entry = state
            .history
            .lock()
            .unwrap()
            .insert_entry(&text, &source)
            .map_err(|error| error.to_string())?;
        let _ = inject::inject_text(&text);
        Some(entry)
    };

    let _ = app.emit(
        "transcription-complete",
        TranscriptionCompletePayload {
            text: text.clone(),
            entry,
        },
    );

    Ok(text)
}

fn show_status(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("status") {
        if let Ok(Some(monitor)) = window
            .current_monitor()
            .or_else(|_| window.primary_monitor())
        {
            if let Ok(size) = window.outer_size() {
                let work_area = monitor.work_area();
                let x = work_area.position.x
                    + ((work_area.size.width.saturating_sub(size.width)) / 2) as i32;
                let y = work_area.position.y
                    + work_area.size.height.saturating_sub(size.height + 26) as i32;
                let _ = window.set_position(PhysicalPosition::new(x, y));
            }
        }
        let _ = window.show();
    }
}

#[tauri::command]
pub fn get_history(state: State<'_, AppState>) -> Result<Vec<TranscriptionEntry>, String> {
    state
        .history
        .lock()
        .unwrap()
        .get_all_entries()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_history_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .history
        .lock()
        .unwrap()
        .delete_entry(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state
        .history
        .lock()
        .unwrap()
        .clear_all()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    mut settings: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if settings.groq_api_key.is_none() {
        settings.groq_api_key = state.config.lock().unwrap().groq_api_key.clone();
    }
    settings.save().map_err(|error| error.to_string())?;
    *state.config.lock().unwrap() = settings.clone();
    reregister_hotkey(&app, &settings.hotkey)?;
    if settings.autostart {
        app.autolaunch()
            .enable()
            .map_err(|error| error.to_string())?;
    } else {
        app.autolaunch()
            .disable()
            .map_err(|error| error.to_string())?;
    }
    let _ = app.emit("settings-updated", settings);
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_groq_api_key(key: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut settings = state.config.lock().unwrap();
    settings.groq_api_key = match key.trim() {
        "" => None,
        value => Some(value.to_string()),
    };
    settings.save().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_groq_api_key(state: State<'_, AppState>) -> String {
    state
        .config
        .lock()
        .unwrap()
        .groq_api_key
        .clone()
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_model_download_status(state: State<'_, AppState>) -> ModelDownloadStatus {
    let model_path = state.config.lock().unwrap().local_model_path.clone();
    let mut status = state.download.lock().unwrap();

    if !status.is_downloading {
        if model_path
            .as_deref()
            .map(|path| std::path::Path::new(path).is_file())
            .unwrap_or(false)
        {
            status.percent = 100.0;
            status.message = "Downloaded".to_string();
        } else if status.percent >= 100.0 {
            status.percent = 0.0;
            status.message = "Not downloaded".to_string();
        }
    }

    status.clone()
}

#[tauri::command]
pub async fn download_local_model(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    {
        let mut status = state.download.lock().unwrap();
        if status.is_downloading {
            return Err("model download is already in progress".to_string());
        }
        status.is_downloading = true;
        status.percent = 0.0;
        status.message = "Starting download".to_string();
    }

    let model_path = {
        let mut settings = state.config.lock().unwrap().clone();
        let path = settings
            .local_model_path
            .clone()
            .unwrap_or_else(|| config::default_model_path().to_string_lossy().to_string());
        settings.local_model_path = Some(path.clone());
        let _ = settings.save();
        *state.config.lock().unwrap() = settings;
        path
    };

    let result = download_model(&app, &state, &model_path).await;
    let mut status = state.download.lock().unwrap();
    status.is_downloading = false;

    match result {
        Ok(()) => {
            status.percent = 100.0;
            status.message = "Downloaded".to_string();
            let _ = app.emit("model-download-progress", status.clone());
            Ok(model_path)
        }
        Err(error) => {
            status.message = error.clone();
            let _ = app.emit("model-download-progress", status.clone());
            Err(error)
        }
    }
}

async fn download_model(
    app: &AppHandle,
    state: &State<'_, AppState>,
    model_path: &str,
) -> Result<(), String> {
    let path = std::path::PathBuf::from(model_path);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }

    let response = reqwest::Client::new()
        .get(MODEL_URL)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    if !response.status().is_success() {
        return Err(format!("model download failed: {}", response.status()));
    }

    let total = response.content_length().unwrap_or(0);
    let mut stream = response.bytes_stream();
    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(|error| error.to_string())?;
    let mut downloaded = 0u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| error.to_string())?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|error| error.to_string())?;
        downloaded += chunk.len() as u64;

        if total > 0 {
            let percent = (downloaded as f64 / total as f64) * 100.0;
            let payload = {
                let mut status = state.download.lock().unwrap();
                status.percent = percent;
                status.message = format!("{percent:.0}% downloaded");
                status.clone()
            };
            let _ = app.emit("model-download-progress", payload);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>, String> {
    audio::list_input_devices().map_err(|error| error.to_string())
}
