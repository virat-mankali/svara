mod groq;
mod local;

use std::path::Path;

use crate::config::{AppConfig, Backend};

pub async fn transcribe(wav_path: &Path, config: &AppConfig) -> anyhow::Result<String> {
    match config.backend {
        Backend::Groq => {
            let api_key = crate::keychain::get_api_key()
                .map_err(|_| anyhow::anyhow!("Groq API key has not been saved yet"))?;
            groq::transcribe_groq(wav_path, &api_key).await
        }
        Backend::Local => {
            let model_path = config
                .local_model_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("local model path is not configured"))?;
            local::transcribe_local(wav_path, model_path).await
        }
    }
}
