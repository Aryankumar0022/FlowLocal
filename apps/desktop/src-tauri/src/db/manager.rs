// ============================================================
// db/manager.rs — SQLite connection pool and migration runner
// ============================================================

use anyhow::{Context, Result};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Tauri managed-state wrapper around the SQLite connection pool.
pub struct DbState {
    pub pool: SqlitePool,
}

/// Resolve the path for the SQLite database file.
/// Uses Tauri's app-local-data directory so the DB is always in a
/// user-specific, writable location.
fn db_path(app: &AppHandle) -> Result<PathBuf> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .context("Failed to resolve app local data directory")?;

    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Failed to create data dir: {}", data_dir.display()))?;

    Ok(data_dir.join("flowlocal.db"))
}

/// Initialize the SQLite pool and run pending migrations.
/// Returns the open pool ready for queries.
pub async fn init(app: &AppHandle) -> Result<SqlitePool> {
    let path = db_path(app)?;
    let url = format!("sqlite://{}?mode=rwc", path.display());

    tracing::info!("Opening database: {}", path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .min_connections(2)
        .connect(&url)
        .await
        .with_context(|| format!("Failed to open SQLite database at {}", path.display()))?;

    // Run embedded SQL migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    // Enable WAL mode and foreign keys — must be done before any queries
    sqlx::query("PRAGMA journal_mode = WAL;")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(pool)
        .await?;
    sqlx::query("PRAGMA synchronous = NORMAL;")
        .execute(pool)
        .await?;

    // Create migrations tracking table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            name       TEXT    NOT NULL UNIQUE,
            applied_at INTEGER NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    let migrations: &[(&str, &str)] = &[
        ("001_initial", include_str!("migrations/001_initial.sql")),
        ("002_dictionary", include_str!("migrations/002_dictionary.sql")),
        ("003_history", include_str!("migrations/003_history.sql")),
        ("004_memory", include_str!("migrations/004_memory.sql")),
        ("005_default_toggle_hotkey", include_str!("migrations/005_default_toggle_hotkey.sql")),
    ];

    for (name, sql) in migrations {
        let already_applied: bool =
            sqlx::query_scalar("SELECT COUNT(*) > 0 FROM _migrations WHERE name = ?")
                .bind(name)
                .fetch_one(pool)
                .await
                .unwrap_or(false);

        if already_applied {
            tracing::debug!("Migration {} already applied, skipping", name);
            continue;
        }

        tracing::info!("Applying migration: {}", name);

        // Execute the SQL (migrations may contain multiple statements)
        // We use `execute` with the full text; SQLite runs them all.
        sqlx::raw_sql(sql).execute(pool).await.with_context(|| {
            format!("Failed to apply migration {}", name)
        })?;

        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query("INSERT INTO _migrations (name, applied_at) VALUES (?, ?)")
            .bind(name)
            .bind(now)
            .execute(pool)
            .await?;

        tracing::info!("Migration {} applied successfully", name);
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Session helpers
// ──────────────────────────────────────────────────────────────

pub struct NewSession {
    pub id: String,
    pub raw_text: String,
    pub clean_text: String,
    pub app_context: String,
    pub window_title: String,
    pub language: String,
    pub duration_ms: i64,
    pub model_used: String,
    pub cleanup_model: String,
}

pub async fn insert_session(pool: &SqlitePool, s: &NewSession) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    let word_count = s.clean_text.split_whitespace().count() as i64;
    let char_count = s.clean_text.len() as i64;

    sqlx::query!(
        "INSERT INTO sessions
            (id, created_at, raw_text, clean_text, app_context, window_title,
             language, duration_ms, model_used, cleanup_model, word_count, char_count)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        s.id,
        now,
        s.raw_text,
        s.clean_text,
        s.app_context,
        s.window_title,
        s.language,
        s.duration_ms,
        s.model_used,
        s.cleanup_model,
        word_count,
        char_count
    )
    .execute(pool)
    .await
    .context("Failed to insert session")?;

    // Also log in history table
    sqlx::query!(
        "INSERT INTO history (session_id, event_type, inserted_at, success)
         VALUES (?, 'dictation', ?, 1)",
        s.id,
        now
    )
    .execute(pool)
    .await
    .context("Failed to insert history entry")?;

    Ok(())
}

/// Fetch recent sessions for the history view.
pub async fn get_sessions(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<SessionRow>> {
    let rows = sqlx::query_as!(
        SessionRowRaw,
        "SELECT id, created_at, raw_text, clean_text, app_context, window_title,
                language, duration_ms, word_count
         FROM sessions
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(pool)
    .await
    .context("Failed to fetch sessions")?;

    Ok(rows
        .into_iter()
        .map(|r| SessionRow {
            id:           r.id.unwrap_or_default(),
            created_at:   r.created_at,
            raw_text:     r.raw_text.unwrap_or_default(),
            clean_text:   r.clean_text.unwrap_or_default(),
            app_context:  r.app_context.unwrap_or_default(),
            window_title: r.window_title.unwrap_or_default(),
            language:     r.language.unwrap_or_default(),
            duration_ms:  r.duration_ms,
            word_count:   r.word_count,
        })
        .collect())
}

pub async fn delete_session(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM sessions WHERE id = ?", id)
        .execute(pool)
        .await
        .context("Failed to delete session")?;
    Ok(())
}

pub async fn clear_all_history(pool: &SqlitePool) -> Result<()> {
    sqlx::query!("DELETE FROM sessions").execute(pool).await?;
    sqlx::query!("DELETE FROM history").execute(pool).await?;
    Ok(())
}

// ──────────────────────────────────────────────────────────────
// Row types returned from queries
// ──────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct SessionRow {
    pub id: String,
    pub created_at: i64,
    pub raw_text: String,
    pub clean_text: String,
    pub app_context: String,
    pub window_title: String,
    pub language: String,
    pub duration_ms: i64,
    pub word_count: i64,
}

// Internal sqlx row that mirrors nullable column types
struct SessionRowRaw {
    id: Option<String>,
    created_at: i64,
    raw_text: Option<String>,
    clean_text: Option<String>,
    app_context: Option<String>,
    window_title: Option<String>,
    language: Option<String>,
    duration_ms: i64,
    word_count: i64,
}

// ──────────────────────────────────────────────────────────────
// Dictionary helpers
// ──────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct DictionaryEntry {
    pub id: i64,
    pub wrong: String,
    pub correct: String,
    pub created_at: i64,
    pub use_count: i64,
}

pub async fn get_dictionary(pool: &SqlitePool) -> Result<Vec<DictionaryEntry>> {
    let rows = sqlx::query_as!(
        DictionaryEntry,
        "SELECT id, wrong, correct, created_at, use_count FROM dictionary ORDER BY use_count DESC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn add_dictionary_entry(pool: &SqlitePool, wrong: &str, correct: &str) -> Result<i64> {
    let now = chrono::Utc::now().timestamp_millis();
    let result = sqlx::query!(
        "INSERT INTO dictionary (wrong, correct, created_at) VALUES (?, ?, ?)
         ON CONFLICT(wrong) DO UPDATE SET correct = excluded.correct, created_at = excluded.created_at",
        wrong,
        correct,
        now
    )
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn delete_dictionary_entry(pool: &SqlitePool, id: i64) -> Result<()> {
    sqlx::query!("DELETE FROM dictionary WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn increment_dictionary_use(pool: &SqlitePool, wrong: &str) -> Result<()> {
    sqlx::query!(
        "UPDATE dictionary SET use_count = use_count + 1 WHERE wrong = ? COLLATE NOCASE",
        wrong
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Returns all dictionary terms as (wrong, correct) pairs for inclusion in LLM prompts.
pub async fn get_dictionary_pairs(pool: &SqlitePool) -> Result<Vec<[String; 2]>> {
    let entries = get_dictionary(pool).await?;
    Ok(entries
        .into_iter()
        .map(|e| [e.wrong, e.correct])
        .collect())
}
