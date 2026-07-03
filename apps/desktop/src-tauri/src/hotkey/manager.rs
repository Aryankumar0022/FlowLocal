// ============================================================
// hotkey/manager.rs — Global hotkey registration
//
// Uses tauri-plugin-global-shortcut.
// Supports two modes (configured in Settings):
//   - Hold:   press to start recording, release to stop
//   - Toggle: press once to start, press again to stop
//
// On press:
//   1. Detects active application context
//   2. Emits "recording:start" Tauri event to React overlay
//   3. Sends AudioCommand::Start to the audio capture thread
//
// On release (hold mode only):
//   1. Sends AudioCommand::Stop
//   2. Emits "recording:processing" Tauri event
// ============================================================

use anyhow::{Context, Result};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use uuid::Uuid;

use crate::audio::{AudioCommand, AudioState};
use crate::context::ContextDetector;
use crate::ipc::IpcBridge;
use crate::settings::SettingsState;

pub struct HotkeyManager;

impl HotkeyManager {
    /// Register the global hotkey from settings.
    /// Must be called once during app setup, after state is managed.
    pub fn register(app: &AppHandle) -> Result<()> {
        let settings_state = app.state::<SettingsState>();
        let hotkey_str = {
            let settings = settings_state
                .inner
                .try_read()
                .context("Failed to read settings for hotkey registration")?;
            settings.hotkey.clone()
        };

        let shortcut = parse_shortcut(&hotkey_str)
            .with_context(|| format!("Invalid hotkey: {}", hotkey_str))?;

        let app_handle = app.clone();

        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                match event.state() {
                    ShortcutState::Pressed => {
                        handle_press(app_handle.clone());
                    }
                    ShortcutState::Released => {
                        handle_release(app_handle.clone());
                    }
                }
            })
            .context("Failed to register global hotkey")?;

        tracing::info!("Global hotkey registered: {}", hotkey_str);
        Ok(())
    }

    /// Unregister the current hotkey (called before re-registering with new key).
    pub fn unregister(app: &AppHandle) -> Result<()> {
        let settings_state = app.state::<SettingsState>();
        let hotkey_str = {
            let settings = settings_state
                .inner
                .try_read()
                .context("Failed to read settings")?;
            settings.hotkey.clone()
        };

        if let Ok(shortcut) = parse_shortcut(&hotkey_str) {
            app.global_shortcut()
                .unregister(shortcut)
                .ok();
        }

        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────
// Public programmatic API (used by audio_commands.rs)
// ──────────────────────────────────────────────────────────────

pub async fn handle_press_programmatic(app: &AppHandle) -> Result<()> {
    handle_press(app.clone());
    Ok(())
}

pub async fn handle_release_programmatic(app: &AppHandle) -> Result<()> {
    stop_recording_internal(app.clone());
    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Event handlers
// ──────────────────────────────────────────────────────────────

fn handle_press(app: AppHandle) {
    let audio_state = app.state::<AudioState>();

    // A second press should stop the current recording. Marking the session as
    // active before the audio thread starts also avoids rapid double-starts.
    if audio_state
        .is_recording
        .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
        stop_recording_internal(app);
        return;
    }

    // Detect active application context BEFORE focus changes
    let active_app = ContextDetector::detect();
    tracing::info!(
        "Hotkey pressed — context: {:?} | window: {}",
        active_app.context,
        active_app.window_title
    );

    let session_id = Uuid::new_v4().to_string();

    // Emit start event to React overlay
    let _ = app.emit("recording:start", serde_json::json!({
        "session_id": session_id,
        "app_context": active_app.context.as_str(),
        "window_title": active_app.window_title,
    }));

    // Start audio capture
    let (chunk_tx, chunk_rx) = tokio::sync::mpsc::channel(256);

    let app_clone = app.clone();
    let session_clone = session_id.clone();
    let active_app_clone = active_app.clone();

    // Forward audio chunks to IPC bridge in a background task
    tauri::async_runtime::spawn(async move {
        forward_audio_to_whisper(app_clone, chunk_rx, session_clone, active_app_clone).await;
    });

    if let Err(e) = audio_state
        .control_tx
        .try_send(AudioCommand::Start {
            session_id,
            chunk_tx,
        })
    {
        audio_state
            .is_recording
            .store(false, std::sync::atomic::Ordering::SeqCst);
        tracing::error!("Failed to start audio capture: {}", e);
        let _ = app.emit("text:error", serde_json::json!({
            "message": format!("Failed to start audio capture: {}", e),
            "code": 500
        }));
    }
}

fn handle_release(app: AppHandle) {
    let settings_state = app.state::<SettingsState>();
    let is_toggle = settings_state
        .inner
        .try_read()
        .map(|s| s.hotkey_mode == crate::settings::HotkeyMode::Toggle)
        .unwrap_or(false);

    if is_toggle {
        return; // In toggle mode, physical release does not stop recording
    }

    stop_recording_internal(app);
}

fn stop_recording_internal(app: AppHandle) {
    let audio_state = app.state::<AudioState>();

    if !audio_state
        .is_recording
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return;
    }

    tracing::info!("Stopping recording");

    // Signal audio thread to stop
    let _ = audio_state.control_tx.try_send(AudioCommand::Stop);

    // Emit processing state to overlay
    let _ = app.emit("recording:processing", ());
}

// ──────────────────────────────────────────────────────────────
// Audio forwarding pipeline
// ──────────────────────────────────────────────────────────────

async fn forward_audio_to_whisper(
    app: AppHandle,
    mut chunk_rx: tokio::sync::mpsc::Receiver<crate::audio::AudioChunk>,
    session_id: String,
    active_app: crate::context::ActiveApp,
) {
    let ipc = app.state::<IpcBridge>();

    while let Some(chunk) = chunk_rx.recv().await {
        if let Err(e) = ipc.send_audio_chunk(&session_id, &chunk).await {
            tracing::error!("Failed to send audio chunk: {}", e);
            break;
        }

        // Forward partial transcripts from Whisper to React
        if let Some(partial) = ipc.try_recv_partial(&session_id).await {
            let _ = app.emit("transcript:partial", serde_json::json!({
                "session_id": session_id,
                "text": partial,
            }));
        }
    }

    // Audio stream ended — signal Whisper service
    if let Err(e) = ipc.send_audio_end(&session_id).await {
        tracing::error!("Failed to send audio end: {}", e);
        return;
    }

    // Await final transcript from Whisper
    match ipc.await_transcript(&session_id).await {
        Ok(transcript) => {
            let _ = app.emit("transcript:final", serde_json::json!({
                "session_id": session_id.clone(),
                "text": transcript.text.clone(),
            }));

            // Run the full AI pipeline
            run_ai_pipeline(app, session_id, transcript.text, active_app, transcript.language, transcript.duration_ms).await;
        }
        Err(e) => {
            tracing::error!("Whisper transcription failed: {}", e);
            let payload = serde_json::json!({
                "message": format!("Transcription failed: {}", e),
                "code": 500
            });
            let _ = app.emit("text:error", payload.clone());
            let _ = app.emit("error", payload);
        }
    }
}

/// Runs the full AI pipeline after transcription:
/// RAG retrieval → LLM cleanup → dictionary → text injection
async fn run_ai_pipeline(
    app: AppHandle,
    session_id: String,
    raw_text: String,
    active_app: crate::context::ActiveApp,
    language: String,
    duration_ms: u64,
) {
    let ipc = app.state::<IpcBridge>();
    let db_state = app.state::<crate::db::DbState>();
    let settings_state = app.state::<SettingsState>();

    let settings = settings_state.inner.read().await;

    // 1. Retrieve RAG context (parallel with settings read — already done)
    let rag_context = if settings.rag_enabled {
        ipc.retrieve_context(&session_id, &raw_text, settings.rag_max_results)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    // 2. Fetch dictionary terms
    let dict_terms = crate::db::manager::get_dictionary_pairs(&db_state.pool)
        .await
        .unwrap_or_default();

    // 3. LLM cleanup
    let clean_text = if settings.cleanup_enabled {
        match ipc
            .clean_text(
                &session_id,
                &raw_text,
                active_app.context.as_str(),
                &settings.language,
                &settings.cleanup_aggressiveness,
                rag_context.clone(),
                dict_terms.clone(),
                settings.remove_fillers,
                settings.fix_punctuation,
                settings.fix_capitalization,
            )
            .await
        {
            Ok(cleaned) => cleaned,
            Err(e) => {
                tracing::warn!("LLM cleanup failed, using raw text: {}", e);
                raw_text.clone()
            }
        }
    } else {
        raw_text.clone()
    };

    drop(settings);

    // 4. Emit cleaned text to React
    let word_count = clean_text.split_whitespace().count();
    let _ = app.emit("text:ready", serde_json::json!({
        "session_id": session_id.clone(),
        "text": clean_text.clone(),
        "raw_text": raw_text.clone(),
        "clean_text": clean_text.clone(),
        "app_context": active_app.context.as_str(),
        "language": language.clone(),
        "duration_ms": duration_ms,
        "word_count": word_count,
    }));

    // 5. Inject into active application
    match crate::inject::inject_text(&clean_text) {
        Ok(_) => {
            let _ = app.emit("text:injected", serde_json::json!({
                "session_id": session_id.clone(),
                "text": clean_text.clone(),
                "app_context": active_app.context.as_str(),
            }));
            tracing::info!("Text injected: {} chars", clean_text.len());
        }
        Err(e) => {
            tracing::error!("Text injection failed: {}", e);
            let payload = serde_json::json!({
                "message": format!("Text injection failed: {}", e),
                "code": 500
            });
            let _ = app.emit("text:error", payload.clone());
            let _ = app.emit("error", payload);
        }
    }

    // 6. Persist session to DB
    let settings_state = app.state::<SettingsState>();
    let model_used = settings_state.inner.read().await.whisper_model.as_str().to_string();
    let ollama_model = settings_state.inner.read().await.ollama_model.clone();

    let session = crate::db::manager::NewSession {
        id: session_id.clone(),
        raw_text,
        clean_text: clean_text.clone(),
        app_context: active_app.context.as_str().to_string(),
        window_title: active_app.window_title,
        language,
        duration_ms: duration_ms as i64,
        model_used,
        cleanup_model: ollama_model,
    };

    if let Err(e) = crate::db::manager::insert_session(&db_state.pool, &session).await {
        tracing::error!("Failed to persist session: {}", e);
    }

    // 7. Store correction in RAG for future retrieval
    let ipc = app.state::<IpcBridge>();
    if let Err(e) = ipc
        .store_correction(
            &session_id,
            &session.raw_text,
            &clean_text,
            active_app.context.as_str(),
            &session.language,
        )
        .await
    {
        tracing::warn!("Failed to store RAG correction: {}", e);
    }
}

// ──────────────────────────────────────────────────────────────
// Hotkey string parser
// ──────────────────────────────────────────────────────────────

/// Parse a hotkey string like "ctrl+space", "alt+shift+f", etc.
fn parse_shortcut(hotkey: &str) -> Result<Shortcut> {
    let lowered = hotkey.to_lowercase();
    let parts: Vec<&str> = lowered
        .split('+')
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .into_iter()
        .collect();

    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in &parts {
        match *part {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "meta" | "super" | "win" | "cmd" | "command" => modifiers |= Modifiers::META,
            key => {
                code = Some(parse_code(key)?);
            }
        }
    }

    let code = code.context("No key code found in hotkey string")?;
    Ok(Shortcut::new(
        if modifiers.is_empty() {
            None
        } else {
            Some(modifiers)
        },
        code,
    ))
}

fn parse_code(key: &str) -> Result<Code> {
    let code = match key {
        "space" => Code::Space,
        "enter" | "return" => Code::Enter,
        "tab" => Code::Tab,
        "backspace" => Code::Backspace,
        "escape" | "esc" => Code::Escape,
        "delete" => Code::Delete,
        "home" => Code::Home,
        "end" => Code::End,
        "pageup" => Code::PageUp,
        "pagedown" => Code::PageDown,
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f10" => Code::F10,
        "f11" => Code::F11,
        "f12" => Code::F12,
        "a" => Code::KeyA, "b" => Code::KeyB, "c" => Code::KeyC, "d" => Code::KeyD,
        "e" => Code::KeyE, "f" => Code::KeyF, "g" => Code::KeyG, "h" => Code::KeyH,
        "i" => Code::KeyI, "j" => Code::KeyJ, "k" => Code::KeyK, "l" => Code::KeyL,
        "m" => Code::KeyM, "n" => Code::KeyN, "o" => Code::KeyO, "p" => Code::KeyP,
        "q" => Code::KeyQ, "r" => Code::KeyR, "s" => Code::KeyS, "t" => Code::KeyT,
        "u" => Code::KeyU, "v" => Code::KeyV, "w" => Code::KeyW, "x" => Code::KeyX,
        "y" => Code::KeyY, "z" => Code::KeyZ,
        "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2,
        "3" => Code::Digit3, "4" => Code::Digit4, "5" => Code::Digit5,
        "6" => Code::Digit6, "7" => Code::Digit7, "8" => Code::Digit8,
        "9" => Code::Digit9,
        _ => anyhow::bail!("Unknown key code: {}", key),
    };
    Ok(code)
}
