// commands/segment.rs — IPC handlers for segment operations

use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use crate::core::models::{Segment, SegmentStatus, SegmentHistory};

/// Get all segments for a file, ordered by seq_number.
#[tauri::command]
pub async fn get_segments(
    file_id: String,
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Segment>, String> {
    state.project_db
        .lock().unwrap()
        .get_segments_for_file(&file_id)
        .map_err(|e| e.to_string())
}

/// Get a single segment by ID.
#[tauri::command]
pub async fn get_segment(
    segment_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Segment>, String> {
    // For now, get all and find — Phase 2 adds get_segment_by_id to ProjectDb
    // This is acceptable for Phase 0 since segments are already loaded in frontend
    Ok(None) // TODO: add get_by_id
}

#[derive(serde::Deserialize)]
pub struct UpdateSegmentArgs {
    pub segment_id: String,
    pub target_text: String,
    pub status: String,
}

/// Save a segment's target text and status (called on every edit for auto-save).
#[tauri::command]
pub async fn update_segment(
    args: UpdateSegmentArgs,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let status = SegmentStatus::from_str(&args.status);
    let db = state.project_db.lock().unwrap();

    db.update_segment_target(&args.segment_id, &args.target_text, &status, &now)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(serde::Deserialize)]
pub struct ConfirmSegmentArgs {
    pub segment_id:   String,
    pub target_text:  String,
    pub project_id:   String,
    pub file_name:    Option<String>,
    pub source_text:  String,
    pub src_lang:     String,
    pub tgt_lang:     String,
}

/// Confirm a segment: set status → Translated, write to history, write-back to TM.
#[tauri::command]
pub async fn confirm_segment(
    args: ConfirmSegmentArgs,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let db = state.project_db.lock().unwrap();

    // Update segment status → Translated
    db.update_segment_target(
        &args.segment_id,
        &args.target_text,
        &SegmentStatus::Translated,
        &now,
    ).map_err(|e| e.to_string())?;

    // Write to history
    let history = SegmentHistory {
        id:          None,
        segment_id:  args.segment_id.clone(),
        target_text: args.target_text.clone(),
        action:      "confirm".into(),
        actor:       None,
        timestamp:   now,
    };
    db.insert_history(&history).map_err(|e| e.to_string())?;

    // TM write-back is handled by the tm command (tm::add_to_tm)
    // so the frontend calls both confirm_segment + add_to_tm on confirmation.

    log::debug!("Confirmed segment: {}", args.segment_id);
    Ok(())
}
