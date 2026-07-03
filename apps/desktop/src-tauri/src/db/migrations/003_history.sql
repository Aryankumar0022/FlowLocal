-- ============================================================
-- Migration 003: Command history and events
-- ============================================================

CREATE TABLE IF NOT EXISTS history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT    NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    event_type  TEXT    NOT NULL CHECK(event_type IN ('dictation', 'command', 'error')),
    command     TEXT,                              -- NULL for dictation events
    inserted_at INTEGER NOT NULL,
    success     INTEGER NOT NULL DEFAULT 1         -- 0 = failed
);

CREATE INDEX IF NOT EXISTS idx_history_session ON history (session_id);
CREATE INDEX IF NOT EXISTS idx_history_type    ON history (event_type);
CREATE INDEX IF NOT EXISTS idx_history_time    ON history (inserted_at DESC);

-- View: recent successful dictations with session info
CREATE VIEW IF NOT EXISTS v_recent_dictations AS
    SELECT
        h.id          AS history_id,
        s.id          AS session_id,
        s.raw_text,
        s.clean_text,
        s.app_context,
        s.window_title,
        s.language,
        s.duration_ms,
        s.word_count,
        h.inserted_at
    FROM history h
    JOIN sessions s ON s.id = h.session_id
    WHERE h.event_type = 'dictation'
      AND h.success = 1
    ORDER BY h.inserted_at DESC;
