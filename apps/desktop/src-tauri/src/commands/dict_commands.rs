// ============================================================
// commands/dict_commands.rs — Personal dictionary CRUD
// ============================================================

use tauri::{command, State};

use crate::db::{manager, DbState};

pub use manager::DictionaryEntry;

/// Get all dictionary entries sorted by use frequency.
#[command]
pub async fn get_dictionary(
    db: State<'_, DbState>,
) -> Result<Vec<DictionaryEntry>, String> {
    manager::get_dictionary(&db.pool)
        .await
        .map_err(|e| e.to_string())
}

/// Add or update a dictionary entry (upsert by `wrong` term).
#[command]
pub async fn add_dictionary_entry(
    db: State<'_, DbState>,
    wrong: String,
    correct: String,
) -> Result<i64, String> {
    if wrong.trim().is_empty() || correct.trim().is_empty() {
        return Err("Both 'wrong' and 'correct' must be non-empty".to_string());
    }
    manager::add_dictionary_entry(&db.pool, wrong.trim(), correct.trim())
        .await
        .map_err(|e| e.to_string())
}

/// Delete a dictionary entry by ID.
#[command]
pub async fn delete_dictionary_entry(
    db: State<'_, DbState>,
    id: i64,
) -> Result<(), String> {
    manager::delete_dictionary_entry(&db.pool, id)
        .await
        .map_err(|e| e.to_string())
}

/// Bulk-import dictionary entries from a JSON array of {wrong, correct} objects.
/// Existing entries for the same `wrong` term are overwritten.
#[command]
pub async fn import_dictionary(
    db: State<'_, DbState>,
    entries: Vec<serde_json::Value>,
) -> Result<usize, String> {
    let mut count = 0;
    for entry in &entries {
        let wrong = entry
            .get("wrong")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let correct = entry
            .get("correct")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        if wrong.is_empty() || correct.is_empty() {
            continue;
        }

        manager::add_dictionary_entry(&db.pool, &wrong, &correct)
            .await
            .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

/// Export all dictionary entries as JSON.
#[command]
pub async fn export_dictionary(
    db: State<'_, DbState>,
) -> Result<Vec<serde_json::Value>, String> {
    let entries = manager::get_dictionary(&db.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(entries
        .into_iter()
        .map(|e| {
            serde_json::json!({
                "wrong": e.wrong,
                "correct": e.correct,
                "use_count": e.use_count,
            })
        })
        .collect())
}
