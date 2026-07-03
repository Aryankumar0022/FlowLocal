// ============================================================
// commands/audio_commands.rs — Tauri IPC commands for recording
// ============================================================

use tauri::{command, AppHandle, Manager, State};
use serde::Serialize;

use crate::audio::{AudioCommand, AudioState};
use crate::ipc::IpcBridge;

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub whisper: bool,
    pub llm: bool,
    pub rag: bool,
    pub is_recording: bool,
}

/// Check health of all AI services and return current recording state.
#[command]
pub async fn get_ai_status(
    audio_state: State<'_, AudioState>,
    ipc: State<'_, IpcBridge>,
) -> Result<ServiceStatus, String> {
    let (whisper, llm, rag) = ipc.health_check().await;
    let is_recording = audio_state
        .is_recording
        .load(std::sync::atomic::Ordering::SeqCst);

    Ok(ServiceStatus {
        whisper,
        llm,
        rag,
        is_recording,
    })
}

/// Retrieve available audio input device names.
#[command]
pub async fn get_audio_devices(
    audio_state: State<'_, AudioState>,
) -> Result<Vec<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    audio_state
        .control_tx
        .send(AudioCommand::ListDevices { reply: tx })
        .await
        .map_err(|e| format!("Failed to request device list: {}", e))?;

    rx.await.map_err(|e| format!("Failed to receive device list: {}", e))
}

/// Programmatic recording start (used by frontend toggle button).
#[command]
pub async fn start_recording(app: AppHandle) -> Result<(), String> {
    crate::hotkey::manager::handle_press_programmatic(&app)
        .await
        .map_err(|e| e.to_string())
}

/// Programmatic recording stop.
#[command]
pub async fn stop_recording(app: AppHandle) -> Result<(), String> {
    crate::hotkey::manager::handle_release_programmatic(&app)
        .await
        .map_err(|e| e.to_string())
}
