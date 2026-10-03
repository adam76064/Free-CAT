// commands/qa.rs — IPC handlers for QA operations

use tauri::State;
use chrono::Utc;

use crate::state::AppState;
use crate::core::models::{QaResult, Segment};
use crate::core::qa::checks::{run_all_checks};
use crate::core::db::tb_db::TbDb;

/// Run all QA checks on every segment of every file in a project.
/// Clears old results first, then re-runs.
#[tauri::command]
pub async fn run_qa(
    project_id: String,
    file_ids: Vec<String>,
    tb_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<QaResult>, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let db = state.project_db.lock().unwrap();

    // Clear previous QA results
    db.clear_qa_results(&project_id).map_err(|e| e.to_string())?;

    // Load forbidden terms from all active termbases
    let tb_index = state.tb_index.lock().unwrap().clone();
    let mut forbidden_terms = Vec::new();
    for entry in tb_index.iter().filter(|e| tb_ids.contains(&e.id)) {
        if let Ok(tb) = TbDb::open(entry.id.clone(), entry.name.clone(), entry.path.clone()) {
            let terms = tb.get_all().unwrap_or_default();
            forbidden_terms.extend(
                terms.into_iter().filter(|t| t.status == "forbidden")
            );
        }
    }

    let mut all_results = Vec::new();

    for file_id in &file_ids {
        let segments = db.get_segments_for_file(file_id)
            .map_err(|e| e.to_string())?;

        for seg in &segments {
            let issues = run_all_checks(seg, &forbidden_terms, None);
            for issue in issues {
                let result = QaResult {
                    id:          None,
                    project_id:  project_id.clone(),
                    segment_id:  Some(seg.id.clone()),
                    check_name:  issue.check_name,
                    severity:    issue.severity,
                    message:     issue.message,
                    run_at:      now.clone(),
                };
                db.insert_qa_result(&result).map_err(|e| e.to_string())?;
                all_results.push(result);
            }
        }
    }

    log::info!("QA run: {} issues found for project {}", all_results.len(), project_id);
    Ok(all_results)
}

/// Get stored QA results for a project.
#[tauri::command]
pub async fn get_qa_results(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<QaResult>, String> {
    state.project_db
        .lock().unwrap()
        .get_qa_results(&project_id)
        .map_err(|e| e.to_string())
}

/// Clear all QA results for a project.
#[tauri::command]
pub async fn clear_qa_results(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.project_db
        .lock().unwrap()
        .clear_qa_results(&project_id)
        .map_err(|e| e.to_string())
}
