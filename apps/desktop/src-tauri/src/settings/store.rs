// ============================================================
// settings/store.rs — Load / save / cache all app settings
// Values are stored as JSON strings in the SQLite `settings` table.
// ============================================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tokio::sync::RwLock;

use super::{CleanupAggressiveness, HotkeyMode, OverlayPosition, Theme, WhisperModel};

// ──────────────────────────────────────────────────────────────
// Settings struct — mirrors the DB defaults in 001_initial.sql
// ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // Hotkey
    pub hotkey: String,
    pub hotkey_mode: HotkeyMode,

    // Models
    pub whisper_model: WhisperModel,
    pub ollama_model: String,
    pub ollama_host: String,
    pub embedding_model: String,

    // Language
    pub language: String,
    pub auto_detect_language: bool,

    // AI Cleanup
    pub cleanup_enabled: bool,
    pub cleanup_aggressiveness: CleanupAggressiveness,
    pub remove_fillers: bool,
    pub fix_punctuation: bool,
    pub fix_capitalization: bool,

    // VAD
    pub vad_enabled: bool,
    pub vad_threshold: f32,
    pub silence_duration_ms: u32,

    // Context
    pub context_aware: bool,
    pub detect_active_app: bool,

    // GPU
    pub use_gpu: bool,
    pub gpu_device_index: i32,

    // RAG
    pub rag_enabled: bool,
    pub rag_max_results: u32,

    // UI
    pub show_overlay: bool,
    pub overlay_position: OverlayPosition,
    pub theme: Theme,

    // Misc
    pub auto_start: bool,
    pub audio_device: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "ctrl+space".to_string(),
            hotkey_mode: HotkeyMode::Toggle,
            whisper_model: WhisperModel::Base,
            ollama_model: "qwen3:4b".to_string(),
            ollama_host: "http://localhost:11434".to_string(),
            embedding_model: "nomic-embed-text".to_string(),
            language: "en".to_string(),
            auto_detect_language: false,
            cleanup_enabled: true,
            cleanup_aggressiveness: CleanupAggressiveness::Moderate,
            remove_fillers: true,
            fix_punctuation: true,
            fix_capitalization: true,
            vad_enabled: true,
            vad_threshold: 0.5,
            silence_duration_ms: 1500,
            context_aware: true,
            detect_active_app: true,
            use_gpu: true,
            gpu_device_index: 0,
            rag_enabled: true,
            rag_max_results: 5,
            show_overlay: true,
            overlay_position: OverlayPosition::Bottom,
            theme: Theme::Dark,
            auto_start: false,
            audio_device: None,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Tauri managed state wrapper
// ──────────────────────────────────────────────────────────────

pub struct SettingsState {
    pub inner: RwLock<Settings>,
}

impl SettingsState {
    pub fn new(settings: Settings) -> Self {
        Self {
            inner: RwLock::new(settings),
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Load from DB (with per-key fallback to defaults)
// ──────────────────────────────────────────────────────────────

pub async fn load(pool: &SqlitePool) -> Result<Settings> {
    let rows = sqlx::query!("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await
        .context("Failed to query settings table")?;

    let map: HashMap<String, String> = rows
        .into_iter()
        .map(|r| (r.key.unwrap_or_default(), r.value))
        .collect();

    let defaults = Settings::default();

    let settings = Settings {
        hotkey: parse_str(&map, "hotkey").unwrap_or(defaults.hotkey),
        hotkey_mode: parse_json(&map, "hotkey_mode").unwrap_or(defaults.hotkey_mode),
        whisper_model: parse_json(&map, "whisper_model").unwrap_or(defaults.whisper_model),
        ollama_model: parse_str(&map, "ollama_model").unwrap_or(defaults.ollama_model),
        ollama_host: parse_str(&map, "ollama_host").unwrap_or(defaults.ollama_host),
        embedding_model: parse_str(&map, "embedding_model").unwrap_or(defaults.embedding_model),
        language: parse_str(&map, "language").unwrap_or(defaults.language),
        auto_detect_language: parse_bool(&map, "auto_detect_language")
            .unwrap_or(defaults.auto_detect_language),
        cleanup_enabled: parse_bool(&map, "cleanup_enabled").unwrap_or(defaults.cleanup_enabled),
        cleanup_aggressiveness: parse_json(&map, "cleanup_aggressiveness")
            .unwrap_or(defaults.cleanup_aggressiveness),
        remove_fillers: parse_bool(&map, "remove_fillers").unwrap_or(defaults.remove_fillers),
        fix_punctuation: parse_bool(&map, "fix_punctuation").unwrap_or(defaults.fix_punctuation),
        fix_capitalization: parse_bool(&map, "fix_capitalization")
            .unwrap_or(defaults.fix_capitalization),
        vad_enabled: parse_bool(&map, "vad_enabled").unwrap_or(defaults.vad_enabled),
        vad_threshold: parse_f32(&map, "vad_threshold").unwrap_or(defaults.vad_threshold),
        silence_duration_ms: parse_u32(&map, "silence_duration_ms")
            .unwrap_or(defaults.silence_duration_ms),
        context_aware: parse_bool(&map, "context_aware").unwrap_or(defaults.context_aware),
        detect_active_app: parse_bool(&map, "detect_active_app")
            .unwrap_or(defaults.detect_active_app),
        use_gpu: parse_bool(&map, "use_gpu").unwrap_or(defaults.use_gpu),
        gpu_device_index: parse_i32(&map, "gpu_device_index").unwrap_or(defaults.gpu_device_index),
        rag_enabled: parse_bool(&map, "rag_enabled").unwrap_or(defaults.rag_enabled),
        rag_max_results: parse_u32(&map, "rag_max_results").unwrap_or(defaults.rag_max_results),
        show_overlay: parse_bool(&map, "show_overlay").unwrap_or(defaults.show_overlay),
        overlay_position: parse_json(&map, "overlay_position").unwrap_or(defaults.overlay_position),
        theme: parse_json(&map, "theme").unwrap_or(defaults.theme),
        auto_start: parse_bool(&map, "auto_start").unwrap_or(defaults.auto_start),
        audio_device: parse_optional_str(&map, "audio_device"),
    };

    Ok(settings)
}

// ──────────────────────────────────────────────────────────────
// Persist a full Settings struct to DB
// ──────────────────────────────────────────────────────────────

pub async fn save(pool: &SqlitePool, settings: &Settings) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();

    macro_rules! upsert {
        ($key:expr, $value:expr) => {
            sqlx::query!(
                "INSERT INTO settings(key, value, updated_at) VALUES (?, ?, ?)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                $key,
                $value,
                now
            )
            .execute(pool)
            .await
            .with_context(|| format!("Failed to save setting: {}", $key))?;
        };
    }

    // Pre-bind all temporary String values so they outlive the sqlx borrow
    let v_hotkey             = json_str(&settings.hotkey);
    let v_hotkey_mode        = serde_json::to_string(&settings.hotkey_mode)?;
    let v_whisper_model      = serde_json::to_string(&settings.whisper_model)?;
    let v_ollama_model       = json_str(&settings.ollama_model);
    let v_ollama_host        = json_str(&settings.ollama_host);
    let v_embedding_model    = json_str(&settings.embedding_model);
    let v_language           = json_str(&settings.language);
    let v_auto_detect        = settings.auto_detect_language.to_string();
    let v_cleanup_enabled    = settings.cleanup_enabled.to_string();
    let v_cleanup_agg        = serde_json::to_string(&settings.cleanup_aggressiveness)?;
    let v_remove_fillers     = settings.remove_fillers.to_string();
    let v_fix_punct          = settings.fix_punctuation.to_string();
    let v_fix_caps           = settings.fix_capitalization.to_string();
    let v_vad_enabled        = settings.vad_enabled.to_string();
    let v_vad_threshold      = settings.vad_threshold.to_string();
    let v_silence_ms         = settings.silence_duration_ms.to_string();
    let v_context_aware      = settings.context_aware.to_string();
    let v_detect_app         = settings.detect_active_app.to_string();
    let v_use_gpu            = settings.use_gpu.to_string();
    let v_gpu_index          = settings.gpu_device_index.to_string();
    let v_rag_enabled        = settings.rag_enabled.to_string();
    let v_rag_max            = settings.rag_max_results.to_string();
    let v_show_overlay       = settings.show_overlay.to_string();
    let v_overlay_pos        = serde_json::to_string(&settings.overlay_position)?;
    let v_theme              = serde_json::to_string(&settings.theme)?;
    let v_auto_start         = settings.auto_start.to_string();
    let v_audio_device       = match &settings.audio_device {
        Some(d) => json_str(d),
        None    => "null".to_string(),
    };

    upsert!("hotkey",                v_hotkey);
    upsert!("hotkey_mode",           v_hotkey_mode);
    upsert!("whisper_model",         v_whisper_model);
    upsert!("ollama_model",          v_ollama_model);
    upsert!("ollama_host",           v_ollama_host);
    upsert!("embedding_model",       v_embedding_model);
    upsert!("language",              v_language);
    upsert!("auto_detect_language",  v_auto_detect);
    upsert!("cleanup_enabled",       v_cleanup_enabled);
    upsert!("cleanup_aggressiveness",v_cleanup_agg);
    upsert!("remove_fillers",        v_remove_fillers);
    upsert!("fix_punctuation",       v_fix_punct);
    upsert!("fix_capitalization",    v_fix_caps);
    upsert!("vad_enabled",           v_vad_enabled);
    upsert!("vad_threshold",         v_vad_threshold);
    upsert!("silence_duration_ms",   v_silence_ms);
    upsert!("context_aware",         v_context_aware);
    upsert!("detect_active_app",     v_detect_app);
    upsert!("use_gpu",               v_use_gpu);
    upsert!("gpu_device_index",      v_gpu_index);
    upsert!("rag_enabled",           v_rag_enabled);
    upsert!("rag_max_results",       v_rag_max);
    upsert!("show_overlay",          v_show_overlay);
    upsert!("overlay_position",      v_overlay_pos);
    upsert!("theme",                 v_theme);
    upsert!("auto_start",            v_auto_start);
    upsert!("audio_device",          v_audio_device);

    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Parsing helpers
// ──────────────────────────────────────────────────────────────

fn parse_str(map: &HashMap<String, String>, key: &str) -> Option<String> {
    let raw = map.get(key)?;
    // Values are stored as JSON strings: `"hello"` → hello
    serde_json::from_str::<String>(raw).ok().or_else(|| {
        // Fallback: the value is a bare string without quotes
        Some(raw.trim_matches('"').to_string())
    })
}

fn parse_optional_str(map: &HashMap<String, String>, key: &str) -> Option<String> {
    let raw = map.get(key)?;
    if raw == "null" {
        return None;
    }
    serde_json::from_str::<String>(raw)
        .ok()
        .or_else(|| Some(raw.trim_matches('"').to_string()))
}

fn parse_bool(map: &HashMap<String, String>, key: &str) -> Option<bool> {
    let raw = map.get(key)?;
    raw.parse::<bool>().ok()
}

fn parse_f32(map: &HashMap<String, String>, key: &str) -> Option<f32> {
    let raw = map.get(key)?;
    raw.parse::<f32>().ok()
}

fn parse_u32(map: &HashMap<String, String>, key: &str) -> Option<u32> {
    let raw = map.get(key)?;
    raw.parse::<u32>().ok()
}

fn parse_i32(map: &HashMap<String, String>, key: &str) -> Option<i32> {
    let raw = map.get(key)?;
    raw.parse::<i32>().ok()
}

fn parse_json<T: for<'de> Deserialize<'de>>(
    map: &HashMap<String, String>,
    key: &str,
) -> Option<T> {
    let raw = map.get(key)?;
    serde_json::from_str(raw).ok()
}

fn json_str(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\\\""))
}
