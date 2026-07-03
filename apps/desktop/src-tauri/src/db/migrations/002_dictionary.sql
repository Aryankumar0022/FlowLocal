-- ============================================================
-- Migration 002: Personal dictionary
-- ============================================================

CREATE TABLE IF NOT EXISTS dictionary (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    wrong      TEXT    NOT NULL,   -- raw / misrecognized form
    correct    TEXT    NOT NULL,   -- correct replacement
    created_at INTEGER NOT NULL,
    use_count  INTEGER NOT NULL DEFAULT 0,
    UNIQUE(wrong COLLATE NOCASE)
);

CREATE INDEX IF NOT EXISTS idx_dictionary_wrong ON dictionary (wrong COLLATE NOCASE);

-- Pre-seed with common tech term corrections
INSERT OR IGNORE INTO dictionary (wrong, correct, created_at) VALUES
    ('land graph',           'LangGraph',      unixepoch() * 1000),
    ('kybernetes',           'Kubernetes',     unixepoch() * 1000),
    ('docker',               'Docker',         unixepoch() * 1000),
    ('pydantic',             'Pydantic',       unixepoch() * 1000),
    ('type script',          'TypeScript',     unixepoch() * 1000),
    ('java script',          'JavaScript',     unixepoch() * 1000),
    ('react js',             'React.js',       unixepoch() * 1000),
    ('git hub',              'GitHub',         unixepoch() * 1000),
    ('pull request',         'pull request',   unixepoch() * 1000),
    ('sequel lite',          'SQLite',         unixepoch() * 1000),
    ('post gress',           'PostgreSQL',     unixepoch() * 1000),
    ('mongo db',             'MongoDB',        unixepoch() * 1000),
    ('fast api',             'FastAPI',        unixepoch() * 1000),
    ('rust lang',            'Rust',           unixepoch() * 1000),
    ('vs code',              'VS Code',        unixepoch() * 1000);
