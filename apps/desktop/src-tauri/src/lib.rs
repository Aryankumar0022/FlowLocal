// ============================================================
// lib.rs — FlowLocal Tauri application entry point
//
// Responsibilities:
//  1. Initialize tracing/logging
//  2. Open SQLite database and run migrations
//  3. Load settings from DB
//  4. Create IPC bridge (connects to Python services)
//  5. Set up system tray
//  6. Register global hotkey
//  7. Register all Tauri IPC command handlers
// ============================================================

mod audio;
mod commands;
mod context;
mod db;
mod hotkey;
mod inject;
mod ipc;
mod settings;
mod tray;
mod services;

use tauri::{AppHandle, Emitter, Manager};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use audio::AudioState;
use db::DbState;
use ipc::IpcBridge;
use settings::{store, SettingsState};
use services::ServiceManager;

pub fn run() {
    // ── Logging ──────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("FlowLocal starting...");

    tauri::Builder::default()
        // ── Plugins ──────────────────────────────────────────
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec![]),
            ),
        )
        .plugin(tauri_plugin_process::init())
        // ── Setup ────────────────────────────────────────────
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Run async setup synchronously (Tauri setup is sync)
            tauri::async_runtime::block_on(async {
                setup_app(&app_handle).await
            })?;

            Ok(())
        })
        // ── IPC Commands ─────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            // Audio / recording
            commands::audio_commands::get_ai_status,
            commands::audio_commands::get_audio_devices,
            commands::audio_commands::start_recording,
            commands::audio_commands::stop_recording,
            // Settings
            commands::settings_commands::get_settings,
            commands::settings_commands::save_settings,
            commands::settings_commands::reset_settings,
            // Dictionary
            commands::dict_commands::get_dictionary,
            commands::dict_commands::add_dictionary_entry,
            commands::dict_commands::delete_dictionary_entry,
            commands::dict_commands::import_dictionary,
            commands::dict_commands::export_dictionary,
            // History
            commands::history_commands::get_history,
            commands::history_commands::delete_session,
            commands::history_commands::clear_history,
            commands::history_commands::reinject_session,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                    let app = window.app_handle();
                    let _ = app.emit("main:visibility", serde_json::json!({ "hidden": true }));

                    if let Some(overlay) = app.get_webview_window("overlay") {
                        let show_overlay = app
                            .try_state::<SettingsState>()
                            .and_then(|state| state.inner.try_read().ok().map(|s| s.show_overlay))
                            .unwrap_or(true);
                        if show_overlay {
                            let _ = overlay.show();
                        }
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("Fatal error while building FlowLocal")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                tracing::info!("Application exiting...");
                let sm = app_handle.state::<ServiceManager>();
                sm.stop();
            }
        });
}

// ──────────────────────────────────────────────────────────────
// Async setup — all initialization in one place
// ──────────────────────────────────────────────────────────────

async fn setup_app(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database...");
    let pool = db::manager::init(app).await?;

    tracing::info!("Loading settings...");
    let settings = store::load(&pool).await?;
    tracing::info!("Hotkey: {} | Model: {}", settings.hotkey, settings.ollama_model);

    tracing::info!("Starting audio capture engine...");
    let audio_state = AudioState::default();

    tracing::info!("Spawning Python services...");
    let service_manager = ServiceManager::new();
    service_manager.start(app, &settings)?;

    tracing::info!("Connecting to Python AI services...");
    let ipc_bridge = IpcBridge::new();

    // Register managed state (order matters — commands may depend on these)
    app.manage(DbState { pool: pool.clone() });
    app.manage(SettingsState::new(settings));
    app.manage(audio_state);
    app.manage(ipc_bridge);
    app.manage(service_manager);

    // System tray
    tracing::info!("Setting up system tray...");
    tray::menu::setup(app)?;

    // Global hotkey
    tracing::info!("Registering global hotkey...");
    hotkey::HotkeyManager::register(app)?;

    // Spawn background health-check task
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        initial_service_health_check(&app_clone).await;
    });

    tracing::info!("FlowLocal ready.");
    Ok(())
}

/// Wait for Python services to come online, then notify the frontend.
async fn initial_service_health_check(app: &AppHandle) {
    use std::time::Duration;

    let ipc = app.state::<IpcBridge>();

    for attempt in 1..=30u32 {
        let (whisper, llm, rag) = ipc.health_check().await;

        if whisper && llm && rag {
            tracing::info!("All AI services online ✓");
            let _ = app.emit("services:ready", serde_json::json!({
                "whisper": true,
                "llm": true,
                "rag": true,
            }));
            return;
        }

        if attempt % 5 == 0 {
            tracing::warn!(
                "Waiting for AI services (attempt {}/30) — whisper:{} llm:{} rag:{}",
                attempt,
                whisper,
                llm,
                rag
            );
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    tracing::error!("AI services did not come online within 30 seconds");
    let (w, l, r) = ipc.health_check().await;
    let _ = app.emit("services:ready", serde_json::json!({
        "whisper": w,
        "llm": l,
        "rag": r,
        "error": "Some services failed to start. Check logs."
    }));
}
