use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Context;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use serde::Serialize;

const TARGET_SAMPLE_RATE: u32 = 16_000;
const MIN_AUDIO_RMS: f32 = 0.00005;
const MIN_AUDIO_PEAK: f32 = 0.001;
const TARGET_PEAK: f32 = 0.85;
const MAX_NORMALIZE_GAIN: f32 = 30.0;
const LIVE_METER_INITIAL_FLOOR: f32 = 0.00025;
const LIVE_METER_MAX_FLOOR: f32 = 0.003;
const LIVE_METER_COMPRESSION_RANGE: f32 = 22.0;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct AudioMeter {
    pub rms: f32,
    pub peak: f32,
    pub level: f32,
    pub noise_floor: f32,
}

pub struct AudioController {
    tx: mpsc::Sender<AudioRequest>,
    meter: Arc<Mutex<AudioMeter>>,
}

enum AudioRequest {
    Start {
        device: Option<String>,
        reply: mpsc::Sender<Result<(), String>>,
    },
    Stop {
        reply: mpsc::Sender<Result<PathBuf, String>>,
    },
    IsRecording {
        reply: mpsc::Sender<bool>,
    },
}

struct RecorderInner {
    stream: Option<Stream>,
    samples: Arc<Mutex<Vec<f32>>>,
    meter: Arc<Mutex<AudioMeter>>,
    sample_rate: u32,
    channels: u16,
    is_recording: bool,
}

impl AudioController {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let meter = Arc::new(Mutex::new(AudioMeter::default()));
        let recorder_meter = Arc::clone(&meter);

        thread::spawn(move || {
            let mut recorder = RecorderInner::new(recorder_meter);
            while let Ok(request) = rx.recv() {
                match request {
                    AudioRequest::Start { device, reply } => {
                        let result = recorder.start(device.as_deref()).map_err(|e| e.to_string());
                        let _ = reply.send(result);
                    }
                    AudioRequest::Stop { reply } => {
                        let result = recorder.stop_to_wav().map_err(|e| e.to_string());
                        let _ = reply.send(result);
                    }
                    AudioRequest::IsRecording { reply } => {
                        let _ = reply.send(recorder.is_recording);
                    }
                }
            }
        });

        Self { tx, meter }
    }

    pub fn is_recording(&self) -> bool {
        let (reply, rx) = mpsc::channel();
        if self.tx.send(AudioRequest::IsRecording { reply }).is_err() {
            return false;
        }
        rx.recv().unwrap_or(false)
    }

    pub fn start(&self, requested_device: Option<&str>) -> anyhow::Result<()> {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(AudioRequest::Start {
                device: requested_device.map(ToOwned::to_owned),
                reply,
            })
            .context("audio thread is not available")?;
        rx.recv()
            .context("audio thread did not return a start result")?
            .map_err(|error| anyhow::anyhow!(error))
    }

    pub fn stop_to_wav(&self) -> anyhow::Result<PathBuf> {
        let (reply, rx) = mpsc::channel();
        self.tx
            .send(AudioRequest::Stop { reply })
            .context("audio thread is not available")?;
        rx.recv()
            .context("audio thread did not return a stop result")?
            .map_err(|error| anyhow::anyhow!(error))
    }

    pub fn meter(&self) -> AudioMeter {
        *self.meter.lock().unwrap()
    }
}

impl RecorderInner {
    fn new(meter: Arc<Mutex<AudioMeter>>) -> Self {
        Self {
            stream: None,
            samples: Arc::new(Mutex::new(Vec::new())),
            meter,
            sample_rate: TARGET_SAMPLE_RATE,
            channels: 1,
            is_recording: false,
        }
    }

    fn start(&mut self, requested_device: Option<&str>) -> anyhow::Result<()> {
        if self.is_recording {
            return Ok(());
        }

        self.samples.lock().unwrap().clear();
        *self.meter.lock().unwrap() = AudioMeter::default();
        let host = cpal::default_host();
        let device = select_input_device(&host, requested_device)?
            .context("no input audio device available")?;
        let supported = device.default_input_config()?;
        let config: StreamConfig = supported.clone().into();
        self.sample_rate = config.sample_rate.0;
        self.channels = config.channels;

        let samples = Arc::clone(&self.samples);
        let meter = Arc::clone(&self.meter);
        let channels = self.channels as usize;
        let err_fn = |err| eprintln!("audio input stream error: {err}");

        let stream = match supported.sample_format() {
            SampleFormat::F32 => device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    push_mono_samples(data.iter().copied(), channels, &samples, &meter)
                },
                err_fn,
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    push_mono_samples(
                        data.iter().map(|value| *value as f32 / i16::MAX as f32),
                        channels,
                        &samples,
                        &meter,
                    )
                },
                err_fn,
                None,
            )?,
            SampleFormat::U16 => device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    push_mono_samples(
                        data.iter()
                            .map(|value| (*value as f32 / u16::MAX as f32) * 2.0 - 1.0),
                        channels,
                        &samples,
                        &meter,
                    )
                },
                err_fn,
                None,
            )?,
            sample_format => anyhow::bail!("unsupported input sample format: {sample_format:?}"),
        };

        stream.play()?;
        self.stream = Some(stream);
        self.is_recording = true;
        Ok(())
    }

    fn stop_to_wav(&mut self) -> anyhow::Result<PathBuf> {
        if !self.is_recording {
            anyhow::bail!("Svara is not recording");
        }

        self.stream.take();
        self.is_recording = false;
        *self.meter.lock().unwrap() = AudioMeter::default();

        let raw = self.samples.lock().unwrap().clone();
        if raw.len() < (self.sample_rate / 4) as usize {
            anyhow::bail!("recording was too short");
        }

        let samples = if self.sample_rate == TARGET_SAMPLE_RATE {
            raw
        } else {
            linear_resample(&raw, self.sample_rate, TARGET_SAMPLE_RATE)
        };

        if !has_audio_signal(&samples) {
            anyhow::bail!(
                "no microphone audio was captured; check Svara's Microphone permission and selected input device"
            );
        }

        let samples = normalize_samples(&samples);
        let path = std::env::temp_dir().join("svara_recording.wav");
        write_wav(&path, &samples)?;
        Ok(path)
    }
}

pub fn list_input_devices() -> anyhow::Result<Vec<String>> {
    let host = cpal::default_host();
    let devices = host.input_devices()?;
    let mut names = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            names.push(name);
        }
    }
    Ok(names)
}

fn select_input_device(
    host: &cpal::Host,
    requested: Option<&str>,
) -> anyhow::Result<Option<cpal::Device>> {
    if let Some(requested) = requested {
        for device in host.input_devices()? {
            if device.name().ok().as_deref() == Some(requested) {
                return Ok(Some(device));
            }
        }
    }
    Ok(host.default_input_device())
}

fn push_mono_samples<I>(
    data: I,
    channels: usize,
    samples: &Arc<Mutex<Vec<f32>>>,
    meter: &Arc<Mutex<AudioMeter>>,
) where
    I: Iterator<Item = f32>,
{
    let mut output = samples.lock().unwrap();
    let frame_channels = channels.max(1);
    let mut frame_sum = 0.0f32;
    let mut frame_len = 0usize;
    let mut sum_squares = 0.0f32;
    let mut peak = 0.0f32;
    let mut frames = 0usize;

    for sample in data {
        frame_sum += sample;
        frame_len += 1;
        if frame_len == frame_channels {
            let mono = (frame_sum / frame_channels as f32).clamp(-1.0, 1.0);
            output.push(mono.clamp(-1.0, 1.0));
            sum_squares += mono * mono;
            peak = peak.max(mono.abs());
            frames += 1;
            frame_sum = 0.0;
            frame_len = 0;
        }
    }

    drop(output);

    if frames > 0 {
        update_live_meter(meter, (sum_squares / frames as f32).sqrt(), peak);
    }
}

fn update_live_meter(meter: &Arc<Mutex<AudioMeter>>, rms: f32, peak: f32) {
    let mut current = meter.lock().unwrap();
    let weighted_signal = (rms * 0.82 + peak * 0.18).max(0.000001);

    if current.noise_floor <= 0.0 {
        current.noise_floor = weighted_signal.clamp(LIVE_METER_INITIAL_FLOOR, LIVE_METER_MAX_FLOOR);
    } else if weighted_signal < current.noise_floor || current.level < 0.08 {
        let floor_target = weighted_signal.clamp(LIVE_METER_INITIAL_FLOOR, LIVE_METER_MAX_FLOOR);
        let floor_smoothing = if weighted_signal < current.noise_floor {
            0.08
        } else {
            0.012
        };
        current.noise_floor += (floor_target - current.noise_floor) * floor_smoothing;
    }

    let floor = current.noise_floor.max(LIVE_METER_INITIAL_FLOOR);
    let voice_signal = (weighted_signal - floor * 1.55).max(0.0);
    let normalized =
        ((voice_signal / floor).ln_1p() / LIVE_METER_COMPRESSION_RANGE.ln_1p()).clamp(0.0, 1.0);
    let normalized = normalized.powf(0.62);

    let smoothing = if normalized > current.level {
        0.7
    } else {
        0.26
    };
    current.level += (normalized - current.level) * smoothing;
    current.rms = rms;
    current.peak = peak.max(current.peak * 0.82);
}

fn linear_resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if samples.is_empty() || from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = (samples.len() as f64 / ratio).ceil() as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = (pos - idx as f64) as f32;
        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }

    out
}

fn signal_metrics(samples: &[f32]) -> (f32, f32) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }

    let mut sum_squares = 0.0f32;
    let mut peak = 0.0f32;

    for sample in samples {
        let amplitude = sample.abs();
        peak = peak.max(amplitude);
        sum_squares += sample * sample;
    }

    let rms = (sum_squares / samples.len() as f32).sqrt();
    (rms, peak)
}

fn has_audio_signal(samples: &[f32]) -> bool {
    let (rms, peak) = signal_metrics(samples);
    rms >= MIN_AUDIO_RMS || peak >= MIN_AUDIO_PEAK
}

fn normalize_samples(samples: &[f32]) -> Vec<f32> {
    let (_, peak) = signal_metrics(samples);
    if peak <= 0.0 {
        return samples.to_vec();
    }

    let gain = (TARGET_PEAK / peak).min(MAX_NORMALIZE_GAIN).max(1.0);
    samples
        .iter()
        .map(|sample| (sample * gain).clamp(-1.0, 1.0))
        .collect()
}

fn write_wav(path: &PathBuf, samples: &[f32]) -> anyhow::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .with_context(|| format!("failed to create {}", path.display()))?;
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(value)?;
    }
    writer.finalize()?;
    Ok(())
}
