// ============================================================
// audio/capture.rs — Microphone capture via CPAL
//
// Design:
//  - Runs in a dedicated std::thread (cpal::Stream is !Send)
//  - Controlled via channels: AudioController sends commands
//  - PCM chunks (f32, native sample rate) are sent to a tokio channel
//    for forwarding to the Python Whisper service
// ============================================================

use anyhow::{Context, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, SupportedStreamConfig,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{mpsc, oneshot};

// ──────────────────────────────────────────────────────────────
// Types
// ──────────────────────────────────────────────────────────────

/// A chunk of raw audio data from the microphone.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// f32 PCM samples, mono (mixed down from multi-channel if needed)
    pub samples: Vec<f32>,
    /// Native sample rate of the capture device (e.g. 44100 or 48000 Hz)
    pub sample_rate: u32,
    /// Monotonically increasing sequence number
    pub seq: u32,
}

/// Commands sent to the audio capture thread.
pub enum AudioCommand {
    Start {
        session_id: String,
        chunk_tx: mpsc::Sender<AudioChunk>,
    },
    Stop,
    /// Request a list of available input device names
    ListDevices {
        reply: oneshot::Sender<Vec<String>>,
    },
}

/// Tauri managed state: the control channel and active-recording flag.
pub struct AudioState {
    pub control_tx: mpsc::Sender<AudioCommand>,
    pub is_recording: Arc<AtomicBool>,
}

impl Default for AudioState {
    fn default() -> Self {
        let (control_tx, control_rx) = mpsc::channel(8);
        let is_recording = Arc::new(AtomicBool::new(false));

        let is_rec_clone = is_recording.clone();
        std::thread::Builder::new()
            .name("flowlocal-audio".to_string())
            .spawn(move || audio_thread(control_rx, is_rec_clone))
            .expect("Failed to spawn audio thread");

        Self {
            control_tx,
            is_recording,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Audio thread — runs for the lifetime of the app
// ──────────────────────────────────────────────────────────────

fn audio_thread(
    mut control_rx: mpsc::Receiver<AudioCommand>,
    is_recording: Arc<AtomicBool>,
) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime for audio thread");

    rt.block_on(async move {
        let mut active_stream: Option<cpal::Stream> = None;
        let mut active_session: Option<String> = None;

        while let Some(cmd) = control_rx.recv().await {
            match cmd {
                AudioCommand::Start {
                    session_id,
                    chunk_tx,
                } => {
                    if active_stream.is_some() {
                        tracing::warn!("Ignoring start request while already recording");
                        continue;
                    }

                    tracing::info!("Audio capture starting for session: {}", session_id);

                    match start_capture_stream(chunk_tx, &session_id) {
                        Ok(stream) => {
                            is_recording.store(true, Ordering::SeqCst);
                            active_stream = Some(stream);
                            active_session = Some(session_id);
                        }
                        Err(e) => {
                            is_recording.store(false, Ordering::SeqCst);
                            tracing::error!("Audio capture error: {}", e);
                        }
                    }
                }

                AudioCommand::Stop => {
                    if let Some(session_id) = active_session.take() {
                        tracing::info!("Audio capture stopped for session: {}", session_id);
                    }

                    // Dropping the CPAL stream stops capture and drops its chunk sender.
                    // That closes the receiver used by the Whisper forwarding task,
                    // which then sends AudioEnd and waits for the final transcript.
                    active_stream.take();
                    is_recording.store(false, Ordering::SeqCst);
                }

                AudioCommand::ListDevices { reply } => {
                    let devices = list_devices();
                    let _ = reply.send(devices);
                }
            }
        }
    });
}

/// Builds and starts a CPAL input stream. The stream runs until dropped.
fn start_capture_stream(
    chunk_tx: mpsc::Sender<AudioChunk>,
    session_id: &str,
) -> Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No default input device found")?;

    let config = device
        .default_input_config()
        .context("No default input config")?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;

    tracing::debug!(
        "Audio device: {} | {} Hz | {} ch | {:?} | session {}",
        device.name().unwrap_or_default(),
        sample_rate,
        channels,
        config.sample_format(),
        session_id,
    );

    let seq = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let stream = build_stream(&device, &config, chunk_tx, seq, channels, sample_rate)?;
    stream.play().context("Failed to start audio stream")?;
    Ok(stream)
}

fn build_stream(
    device: &Device,
    config: &SupportedStreamConfig,
    chunk_tx: mpsc::Sender<AudioChunk>,
    seq: Arc<std::sync::atomic::AtomicU32>,
    channels: usize,
    sample_rate: u32,
) -> Result<cpal::Stream> {
    let err_fn = |e| tracing::error!("Audio stream error: {}", e);

    let stream = match config.sample_format() {
        SampleFormat::F32 => {
            let tx = chunk_tx.clone();
            let s = seq.clone();
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _| {
                    send_chunk(data, channels, sample_rate, &tx, &s);
                },
                err_fn,
                None,
            )?
        }

        SampleFormat::I16 => {
            let tx = chunk_tx.clone();
            let s = seq.clone();
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[i16], _| {
                    let floats: Vec<f32> =
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                    send_chunk(&floats, channels, sample_rate, &tx, &s);
                },
                err_fn,
                None,
            )?
        }

        SampleFormat::U16 => {
            let tx = chunk_tx.clone();
            let s = seq.clone();
            device.build_input_stream(
                &config.clone().into(),
                move |data: &[u16], _| {
                    let floats: Vec<f32> = data
                        .iter()
                        .map(|&s| (s as f32 - 32768.0) / 32768.0)
                        .collect();
                    send_chunk(&floats, channels, sample_rate, &tx, &s);
                },
                err_fn,
                None,
            )?
        }

        fmt => {
            anyhow::bail!("Unsupported audio sample format: {:?}", fmt);
        }
    };

    Ok(stream)
}

/// Mix down to mono if needed, then send the chunk over the channel.
fn send_chunk(
    data: &[f32],
    channels: usize,
    sample_rate: u32,
    tx: &mpsc::Sender<AudioChunk>,
    seq: &Arc<std::sync::atomic::AtomicU32>,
) {
    let samples: Vec<f32> = if channels == 1 {
        data.to_vec()
    } else {
        // Mix down: average across channels
        data.chunks_exact(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    let current_seq = seq.fetch_add(1, Ordering::Relaxed);

    let chunk = AudioChunk {
        samples,
        sample_rate,
        seq: current_seq,
    };

    // Non-blocking send — drop chunk if channel is full (backpressure)
    if tx.try_send(chunk).is_err() {
        tracing::warn!("Audio chunk dropped (channel full) — seq {}", current_seq);
    }
}

/// List available audio input devices.
pub fn list_devices() -> Vec<String> {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => devices
            .filter_map(|d| d.name().ok())
            .collect(),
        Err(e) => {
            tracing::error!("Failed to enumerate audio devices: {}", e);
            vec![]
        }
    }
}
