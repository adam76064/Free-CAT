// commands/tqa.rs — IPC handlers for Translation Quality Assessment (DQF-MQM)
use tauri::State;
use chrono::Utc;
use crate::state::AppState;
use crate::core::models::{TqaError, TqaScorecard};

#[derive(serde::Deserialize)]
pub struct AddTqaErrorArgs {
    pub project_id: String,
    pub segment_id: String,
    pub category:   String, // "accuracy", "fluency", "terminology", "style", "design"
    pub sub_type:   Option<String>,
    pub severity:   String, // "minor", "major", "critical"
    pub comment:    Option<String>,
}

/// Log a new TQA / MQM error against a segment.
#[tauri::command]
pub async fn add_tqa_error(
    args: AddTqaErrorArgs,
    state: State<'_, AppState>,
) -> Result<TqaError, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let penalty = match args.severity.to_lowercase().as_str() {
        "minor" => 1,
        "major" => 5,
        "critical" => 25,
        _ => 1,
    };

    let error_record = TqaError {
        id: None,
        project_id: args.project_id,
        segment_id: args.segment_id,
        category: args.category,
        sub_type: args.sub_type,
        severity: args.severity,
        penalty,
        comment: args.comment,
        created_at: now,
    };

    let db = state.project_db.lock().unwrap();
    let inserted_id = db.insert_tqa_error(&error_record)
        .map_err(|e| e.to_string())?;

    let mut created = error_record;
    created.id = Some(inserted_id);
    Ok(created)
}

/// Delete a TQA error record by ID.
#[tauri::command]
pub async fn delete_tqa_error(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.project_db.lock().unwrap();
    db.delete_tqa_error(id)
        .map_err(|e| e.to_string())
}

/// Get all TQA errors logged against a specific segment.
#[tauri::command]
pub async fn get_segment_tqa_errors(
    segment_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<TqaError>, String> {
    let db = state.project_db.lock().unwrap();
    db.get_tqa_errors_for_segment(&segment_id)
        .map_err(|e| e.to_string())
}

/// Compute the live TAUS DQF-MQM Quality Scorecard for a project.
#[tauri::command]
pub async fn get_tqa_scorecard(
    project_id: String,
    total_words: Option<i64>,
    state: State<'_, AppState>,
) -> Result<TqaScorecard, String> {
    let words = total_words.unwrap_or(0);
    let db = state.project_db.lock().unwrap();
    db.get_project_tqa_scorecard(&project_id, words)
        .map_err(|e| e.to_string())
}
