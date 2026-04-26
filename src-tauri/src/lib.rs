mod audio;
mod commands;
mod config;
mod history;
mod inject;
mod transcribe;

use std::str::FromStr;
use std::sync::Mutex;

use anyhow::Context;
use audio::AudioController;
use config::AppConfig;
use history::HistoryDb;
use serde::Serialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct AppState {
    pub recorder: AudioController,
    pub config: Mutex<AppConfig>,
    pub history: Mutex<HistoryDb>,
    pub download: Mutex<ModelDownloadStatus>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDownloadStatus {
    pub percent: f64,
    pub is_downloading: bool,
    pub message: String,
}

impl Default for ModelDownloadStatus {
    fn default() -> Self {
        Self {
            percent: 0.0,
            is_downloading: false,
            message: "Not started".to_string(),
        }
    }
}

impl AppState {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            recorder: AudioController::new(),
            config: Mutex::new(AppConfig::load()?),
            history: Mutex::new(HistoryDb::new()?),
            download: Mutex::new(ModelDownloadStatus::default()),
        })
    }
}

fn show_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Open Settings", true, None::<&str>)?;
    let history = MenuItem::with_id(app, "history", "Open History", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &history, &quit])?;

    let mut builder = TrayIconBuilder::with_id("svara")
        .tooltip("Svara")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => show_window(app, "settings"),
            "history" => show_window(app, "history"),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window(tray.app_handle(), "settings");
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

fn register_hotkey(app: &AppHandle, hotkey: &str) -> anyhow::Result<()> {
    let shortcut =
        Shortcut::from_str(hotkey).with_context(|| format!("could not parse hotkey: {hotkey}"))?;
    app.global_shortcut().unregister_all()?;
    app.global_shortcut().register(shortcut)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new().expect("failed to initialize Svara state");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = commands::toggle_recording(app).await;
                        });
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            setup_tray(app.handle())?;

            let state = app.state::<AppState>();
            let hotkey = state.config.lock().unwrap().hotkey.clone();
            if let Err(error) = register_hotkey(app.handle(), &hotkey) {
                eprintln!("failed to register Svara hotkey: {error:#}");
            }

            show_window(app.handle(), "settings");
            Ok(())
        })
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::toggle_recording_command,
            commands::get_history,
            commands::delete_history_entry,
            commands::clear_history,
            commands::save_settings,
            commands::get_settings,
            commands::set_groq_api_key,
            commands::get_groq_api_key,
            commands::download_local_model,
            commands::get_model_download_status,
            commands::list_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Svara");
}

pub fn reregister_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    register_hotkey(app, hotkey).map_err(|error| error.to_string())
}
