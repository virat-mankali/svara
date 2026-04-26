use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GroqResponse {
    text: String,
}

pub async fn transcribe_groq(wav_path: &Path, api_key: &str) -> anyhow::Result<String> {
    let file_bytes = tokio::fs::read(wav_path).await?;
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_bytes)
                .file_name("audio.wav")
                .mime_str("audio/wav")?,
        )
        .text("model", "whisper-large-v3-turbo")
        .text("response_format", "json")
        .text("temperature", "0")
        .text(
            "prompt",
            "Transcribe only clear spoken words. If there is no clear speech, return nothing.",
        );

    let response = reqwest::Client::new()
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Groq transcription failed ({status}): {body}");
    }

    let body: GroqResponse = response.json().await?;
    Ok(body.text.trim().to_string())
}
