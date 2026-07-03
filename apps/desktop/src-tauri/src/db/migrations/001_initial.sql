-- ============================================================
-- Migration 001: Core schema
-- Sessions, settings, and app metadata
-- ============================================================

PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;

-- -------------------------------------------------------
-- Sessions: every dictation/command run is recorded
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS sessions (
    id            TEXT    PRIMARY KEY,           -- UUID v4
    created_at    INTEGER NOT NULL,              -- Unix epoch milliseconds
    raw_text      TEXT    NOT NULL DEFAULT '',
    clean_text    TEXT    NOT NULL DEFAULT '',
    app_context   TEXT    NOT NULL DEFAULT '',   -- 'vscode' | 'slack' | 'email' | ...
    window_title  TEXT    NOT NULL DEFAULT '',
    language      TEXT    NOT NULL DEFAULT 'en', -- ISO 639-1
    duration_ms   INTEGER NOT NULL DEFAULT 0,    -- recording length
    model_used    TEXT    NOT NULL DEFAULT '',   -- e.g. 'base.en'
    cleanup_model TEXT    NOT NULL DEFAULT '',   -- e.g. 'qwen3:4b'
    word_count    INTEGER NOT NULL DEFAULT 0,
    char_count    INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_app     ON sessions (app_context);

-- -------------------------------------------------------
-- Settings: key-value store for all app configuration
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS settings (
    key        TEXT    PRIMARY KEY,
    value      TEXT    NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Insert defaults on first run
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES
    ('hotkey',                      '"ctrl+space"',  unixepoch() * 1000),
    ('hotkey_mode',                 '"toggle"',         unixepoch() * 1000),
    ('whisper_model',               '"base"',         unixepoch() * 1000),
    ('ollama_model',                '"qwen3:4b"',     unixepoch() * 1000),
    ('ollama_host',                 '"http://localhost:11434"', unixepoch() * 1000),
    ('embedding_model',             '"nomic-embed-text"', unixepoch() * 1000),
    ('language',                    '"en"',           unixepoch() * 1000),
    ('auto_detect_language',        'false',          unixepoch() * 1000),
    ('cleanup_enabled',             'true',           unixepoch() * 1000),
    ('cleanup_aggressiveness',      '"moderate"',     unixepoch() * 1000),
    ('remove_fillers',              'true',           unixepoch() * 1000),
    ('fix_punctuation',             'true',           unixepoch() * 1000),
    ('fix_capitalization',          'true',           unixepoch() * 1000),
    ('vad_enabled',                 'true',           unixepoch() * 1000),
    ('vad_threshold',               '0.5',            unixepoch() * 1000),
    ('silence_duration_ms',         '1500',           unixepoch() * 1000),
    ('context_aware',               'true',           unixepoch() * 1000),
    ('detect_active_app',           'true',           unixepoch() * 1000),
    ('use_gpu',                     'true',           unixepoch() * 1000),
    ('gpu_device_index',            '0',              unixepoch() * 1000),
    ('rag_enabled',                 'true',           unixepoch() * 1000),
    ('rag_max_results',             '5',              unixepoch() * 1000),
    ('show_overlay',                'true',           unixepoch() * 1000),
    ('overlay_position',            '"bottom"',       unixepoch() * 1000),
    ('theme',                       '"dark"',         unixepoch() * 1000),
    ('auto_start',                  'false',          unixepoch() * 1000),
    ('audio_device',                'null',           unixepoch() * 1000);
