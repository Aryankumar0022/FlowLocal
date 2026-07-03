// ============================================================
// commands/settings_commands.rs — Tauri IPC commands for settings
// ============================================================

use tauri::{command, AppHandle, Manager, State};

use crate::db::DbState;
use crate::hotkey::HotkeyManager;
use crate::settings::{store, Settings, SettingsState};

/// Return the current settings as a JSON-serializable struct.
#[command]
pub async fn get_settings(
    settings_state: State<'_, SettingsState>,
) -> Result<Settings, String> {
    let settings = settings_state.inner.read().await;
    Ok(settings.clone())
}

/// Save new settings to the database and update in-memory cache.
/// Re-registers the global hotkey if it changed.
#[command]
pub async fn save_settings(
    app: AppHandle,
    new_settings: Settings,
    settings_state: State<'_, SettingsState>,
    db_state: State<'_, DbState>,
) -> Result<(), String> {
    let old_hotkey = {
        let s = settings_state.inner.read().await;
        s.hotkey.clone()
    };

    // Persist to DB
    store::save(&db_state.pool, &new_settings)
        .await
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    // Update cache
    {
        let mut s = settings_state.inner.write().await;
        *s = new_settings.clone();
    }

    // Re-register hotkey if it changed
    if old_hotkey != new_settings.hotkey {
        HotkeyManager::unregister(&app).map_err(|e| e.to_string())?;
        HotkeyManager::register(&app).map_err(|e| e.to_string())?;
    }

    // Handle auto-start setting
    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_autostart::ManagerExt;
        if new_settings.auto_start {
            app.autolaunch().enable().map_err(|e| e.to_string())?;
        } else {
            app.autolaunch().disable().map_err(|e| e.to_string())?;
        }
    }

    tracing::info!("Settings saved");
    Ok(())
}

/// Reset all settings to defaults.
#[command]
pub async fn reset_settings(
    settings_state: State<'_, SettingsState>,
    db_state: State<'_, DbState>,
) -> Result<Settings, String> {
    let defaults = Settings::default();
    store::save(&db_state.pool, &defaults)
        .await
        .map_err(|e| format!("Failed to reset settings: {}", e))?;

    {
        let mut s = settings_state.inner.write().await;
        *s = defaults.clone();
    }

    Ok(defaults)
}
