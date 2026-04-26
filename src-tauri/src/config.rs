use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Groq,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub backend: Backend,
    pub hotkey: String,
    pub audio_device: Option<String>,
    pub local_model_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groq_api_key: Option<String>,
    pub autostart: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            backend: Backend::Groq,
            hotkey: "CmdOrCtrl+Shift+Space".to_string(),
            audio_device: None,
            local_model_path: Some(default_model_path().to_string_lossy().to_string()),
            groq_api_key: None,
            autostart: false,
        }
    }
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut config: Self = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        if config.local_model_path.is_none() {
            config.local_model_path = Some(default_model_path().to_string_lossy().to_string());
        }

        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

pub fn app_dir() -> anyhow::Result<PathBuf> {
    let base = dirs::data_dir().context("could not resolve application data directory")?;
    Ok(base.join("svara"))
}

pub fn config_path() -> anyhow::Result<PathBuf> {
    Ok(app_dir()?.join("config.json"))
}

pub fn default_model_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("svara")
        .join("models")
        .join("ggml-large-v3-turbo.bin")
}
