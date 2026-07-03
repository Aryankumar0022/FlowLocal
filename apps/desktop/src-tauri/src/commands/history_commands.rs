// ============================================================
// commands/history_commands.rs — Session history commands
// ============================================================

use serde::Serialize;
use tauri::{command, State};

use crate::db::{manager, DbState};

pub use manager::SessionRow;

#[derive(Debug, Serialize)]
pub struct HistoryPage {
    pub items: Vec<SessionRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

/// Fetch paginated dictation history.
#[command]
pub async fn get_history(
    db: State<'_, DbState>,
    #[allow(unused_variables)]
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<HistoryPage, String> {
    let limit = limit.unwrap_or(50).min(200);
    let offset = offset.unwrap_or(0).max(0);

    let items = manager::get_sessions(&db.pool, limit, offset)
        .await
        .map_err(|e| e.to_string())?;

    let total: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM sessions")
        .fetch_one(&db.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(HistoryPage {
        items,
        total,
        limit,
        offset,
    })
}

/// Delete a single session (and its history entries via CASCADE).
#[command]
pub async fn delete_session(
    db: State<'_, DbState>,
    id: String,
) -> Result<(), String> {
    manager::delete_session(&db.pool, &id)
        .await
        .map_err(|e| e.to_string())
}

/// Delete all history and sessions.
#[command]
pub async fn clear_history(
    db: State<'_, DbState>,
) -> Result<(), String> {
    manager::clear_all_history(&db.pool)
        .await
        .map_err(|e| e.to_string())
}

/// Re-inject a past session's clean text into the active application.
#[command]
pub async fn reinject_session(
    db: State<'_, DbState>,
    id: String,
) -> Result<(), String> {
    // Fetch clean text for this session
    let session = sqlx::query!(
        "SELECT clean_text FROM sessions WHERE id = ?",
        id
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Session {} not found", id))?;

    crate::inject::inject_text(&session.clean_text).map_err(|e| e.to_string())
}
