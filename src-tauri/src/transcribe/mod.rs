mod groq;
mod local;

use std::path::Path;

use crate::config::{AppConfig, Backend};

pub async fn transcribe(wav_path: &Path, config: &AppConfig) -> anyhow::Result<String> {
    match config.backend {
        Backend::Groq => {
            let api_key = config
                .groq_api_key
                .as_deref()
                .filter(|key| !key.trim().is_empty())
                .ok_or_else(|| anyhow::anyhow!("Groq API key has not been saved yet"))?;
            groq::transcribe_groq(wav_path, &api_key).await
        }
        Backend::Local => {
            let model_path = config
                .local_model_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("local model path is not configured"))?;
            if !std::path::Path::new(model_path).exists() {
                anyhow::bail!(
                    "Local Whisper model is not downloaded yet. Download it from Settings, then try local transcription again."
                );
            }
            local::transcribe_local(wav_path, model_path).await
        }
    }
}
