use std::path::Path;

#[cfg(feature = "local-whisper")]
pub async fn transcribe_local(wav_path: &Path, model_path: &str) -> anyhow::Result<String> {
    let wav_path = wav_path.to_path_buf();
    let model_path = model_path.to_string();

    tauri::async_runtime::spawn_blocking(move || {
        use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

        let mut reader = hound::WavReader::open(&wav_path)?;
        let samples = reader
            .samples::<i16>()
            .map(|sample| sample.map(|value| value as f32 / i16::MAX as f32))
            .collect::<Result<Vec<_>, _>>()?;

        let ctx = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())?;
        let mut state = ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state.full(params, &samples)?;
        let segments = state.full_n_segments()?;
        let mut text = String::new();

        for index in 0..segments {
            text.push_str(state.full_get_segment_text(index)?.trim());
            text.push(' ');
        }

        Ok(text.trim().to_string())
    })
    .await?
}

#[cfg(not(feature = "local-whisper"))]
pub async fn transcribe_local(_wav_path: &Path, _model_path: &str) -> anyhow::Result<String> {
    anyhow::bail!(
        "Local Whisper support is not compiled into this build. Rebuild with `cargo tauri build --features local-whisper`."
    )
}
