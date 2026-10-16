// commands/segment.rs — IPC handlers for segment operations

use tauri::State;
use chrono::Utc;

use crate::state::AppState;
use crate::core::models::{Segment, SegmentStatus, SegmentHistory};

/// Get all segments for a file, ordered by seq_number.
#[tauri::command]
pub async fn get_segments(
    file_id: String,
    _project_id: String,
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
    state.project_db
        .lock().unwrap()
        .get_segment_by_id(&segment_id)
        .map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct SplitSegmentArgs {
    pub segment_id: String,
    pub split_index: usize,
}

#[tauri::command]
pub async fn split_segment(
    args: SplitSegmentArgs,
    state: State<'_, AppState>,
) -> Result<Vec<Segment>, String> {
    let db = state.project_db.lock().unwrap();
    let (seg1, seg2) = db.split_segment(&args.segment_id, args.split_index)
        .map_err(|e| e.to_string())?;
    Ok(vec![seg1, seg2])
}

#[derive(serde::Deserialize)]
pub struct MergeSegmentsArgs {
    pub segment_id_1: String,
    pub segment_id_2: String,
}

#[tauri::command]
pub async fn merge_segments(
    args: MergeSegmentsArgs,
    state: State<'_, AppState>,
) -> Result<Segment, String> {
    let db = state.project_db.lock().unwrap();
    db.merge_segments(&args.segment_id_1, &args.segment_id_2)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_bookmark(
    segment_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    let db = state.project_db.lock().unwrap();
    db.toggle_bookmark(&segment_id)
        .map_err(|e| e.to_string())
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

/// Confirm a segment: set status → Translated, write to history, auto-propagate to repetitions.
#[tauri::command]
pub async fn confirm_segment(
    args: ConfirmSegmentArgs,
    state: State<'_, AppState>,
) -> Result<usize, String> {
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
        timestamp:   now.clone(),
    };
    db.insert_history(&history).map_err(|e| e.to_string())?;

    // Auto-propagate to repeated segments if enabled in project settings
    let mut propagated = 0;
    if let Ok(Some(proj)) = db.get_project(&args.project_id) {
        let settings: crate::core::models::ProjectSettings = serde_json::from_str(&proj.settings)
            .unwrap_or_default();
        if settings.auto_propagate {
            if let Ok(count) = db.propagate_translation(
                &args.project_id,
                &args.source_text,
                &args.target_text,
                &now,
            ) {
                propagated = count;
                if propagated > 0 {
                    log::info!("Auto-propagated translation to {} repetitions", propagated);
                }
            }
        }
    }

    log::debug!("Confirmed segment: {}", args.segment_id);
    Ok(propagated)
}

#[derive(serde::Deserialize)]
pub struct UpdateCommentArgs {
    pub segment_id: String,
    pub comment: Option<String>,
}

#[tauri::command]
pub async fn update_segment_comment(
    args: UpdateCommentArgs,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.project_db.lock().unwrap();
    db.update_segment_comment(&args.segment_id, args.comment.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_segment_history(
    segment_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SegmentHistory>, String> {
    let db = state.project_db.lock().unwrap();
    db.get_history_for_segment(&segment_id)
        .map_err(|e| e.to_string())
}
