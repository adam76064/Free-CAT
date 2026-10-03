// commands/analysis.rs — Word count and TM leverage analysis

use tauri::State;
use crate::state::AppState;
use crate::core::models::{AnalysisReport, SegmentStatus};

/// Analyze a project: compute word counts and TM leverage statistics.
#[tauri::command]
pub async fn analyze_project(
    project_id: String,
    file_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<AnalysisReport, String> {
    let db = state.project_db.lock().unwrap();

    let mut report = AnalysisReport {
        project_id:     project_id.clone(),
        total_words:    0,
        total_segments: 0,
        exact_words:    0,
        fuzzy_words:    0,
        new_words:      0,
        exact_segments: 0,
        fuzzy_segments: 0,
        new_segments:   0,
        tm_savings_pct: 0.0,
    };

    for file_id in &file_ids {
        let segments = db.get_segments_for_file(file_id)
            .map_err(|e| e.to_string())?;

        for seg in &segments {
            let word_count = count_words(&seg.source_text) as i64;
            report.total_segments += 1;
            report.total_words += word_count;

            match seg.match_percent {
                Some(pct) if pct >= 100 => {
                    report.exact_words    += word_count;
                    report.exact_segments += 1;
                }
                Some(pct) if pct >= 75 => {
                    report.fuzzy_words    += word_count;
                    report.fuzzy_segments += 1;
                }
                _ => {
                    report.new_words    += word_count;
                    report.new_segments += 1;
                }
            }
        }
    }

    // TM savings % = (exact + 50% of fuzzy) / total
    if report.total_words > 0 {
        let savings = report.exact_words as f64 + (report.fuzzy_words as f64 * 0.5);
        report.tm_savings_pct = (savings / report.total_words as f64 * 100.0).round();
    }

    Ok(report)
}

/// Get the analysis report for a project (returns same as analyze_project, alias).
#[tauri::command]
pub async fn get_analysis_report(
    project_id: String,
    file_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<AnalysisReport, String> {
    analyze_project(project_id, file_ids, state).await
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}
