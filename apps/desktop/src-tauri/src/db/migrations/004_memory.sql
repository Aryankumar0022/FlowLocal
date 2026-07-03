-- ============================================================
-- Migration 004: Memory — snippets and writing patterns
-- ============================================================

-- -------------------------------------------------------
-- Snippets: user-defined text expansions
-- e.g. trigger "myemail" → "john.doe@example.com"
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS snippets (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    trigger    TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    expansion  TEXT    NOT NULL,
    use_count  INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_snippets_trigger ON snippets (trigger COLLATE NOCASE);

-- -------------------------------------------------------
-- Writing patterns: frequently used phrases
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS writing_patterns (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern     TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    frequency   INTEGER NOT NULL DEFAULT 1,
    last_seen   INTEGER NOT NULL,
    app_context TEXT    NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_patterns_freq ON writing_patterns (frequency DESC);
CREATE INDEX IF NOT EXISTS idx_patterns_app  ON writing_patterns (app_context);

-- -------------------------------------------------------
-- Acronyms: user-defined expansions for spoken acronyms
-- -------------------------------------------------------
CREATE TABLE IF NOT EXISTS acronyms (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    spoken     TEXT    NOT NULL UNIQUE COLLATE NOCASE, -- "rag"
    expanded   TEXT    NOT NULL,                        -- "Retrieval Augmented Generation"
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_acronyms_spoken ON acronyms (spoken COLLATE NOCASE);
