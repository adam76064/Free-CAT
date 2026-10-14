// commands/analysis.rs — Word count and TM leverage analysis

use tauri::State;
use chrono::Utc;
use crate::state::AppState;
use crate::core::models::{AnalysisReport, ProjectSettings, SegmentStatus};
use crate::core::db::tm_db::TmDb;

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

#[derive(serde::Deserialize)]
pub struct PreTranslateArgs {
    pub project_id: String,
    pub file_ids:   Vec<String>,
    pub min_score:  Option<f64>,
    pub overwrite:  Option<bool>,
}

#[derive(serde::Serialize)]
pub struct PreTranslateResult {
    pub total_segments: usize,
    pub pretranslated:  usize,
    pub words_filled:   usize,
}

/// Batch pre-translate untranslated segments using project translation memories.
#[tauri::command]
pub async fn pre_translate(
    args: PreTranslateArgs,
    state: State<'_, AppState>,
) -> Result<PreTranslateResult, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let min_score = args.min_score.unwrap_or(1.0);
    let overwrite = args.overwrite.unwrap_or(false);

    let (src_lang, tgt_lang, tm_ids) = {
        let db = state.project_db.lock().unwrap();
        let proj = db.get_project(&args.project_id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project {} not found", args.project_id))?;
        let settings: ProjectSettings = serde_json::from_str(&proj.settings).unwrap_or_default();
        (proj.src_lang, proj.tgt_lang, settings.tm_ids)
    };

    if tm_ids.is_empty() {
        return Ok(PreTranslateResult {
            total_segments: 0,
            pretranslated: 0,
            words_filled: 0,
        });
    }

    // Open TMs
    let tm_entries = state.tm_index.lock().unwrap().clone();
    let mut open_tms = Vec::new();
    for entry in tm_entries.iter().filter(|e| tm_ids.contains(&e.id)) {
        if let Ok(tm) = TmDb::open(entry.id.clone(), entry.name.clone(), entry.path.clone()) {
            open_tms.push(tm);
        }
    }

    let mut total_segments = 0;
    let mut pretranslated = 0;
    let mut words_filled = 0;

    let db = state.project_db.lock().unwrap();
    for file_id in &args.file_ids {
        let segments = db.get_segments_for_file(file_id)
            .map_err(|e| e.to_string())?;

        for seg in segments {
            total_segments += 1;
            let is_untranslated = seg.target_text.trim().is_empty()
                || seg.status == SegmentStatus::New
                || seg.status == SegmentStatus::Draft;

            if !overwrite && !is_untranslated {
                continue;
            }

            // Look for best match in TMs
            let mut best_match: Option<crate::core::models::TmMatchResult> = None;
            let min_pct = (min_score * 100.0).round() as u8;
            for tm in &open_tms {
                if let Ok(matches) = tm.query(&seg.source_text, &src_lang, &tgt_lang, min_score, 1, 0.0) {
                    if let Some(m) = matches.into_iter().next() {
                        if best_match.as_ref().map_or(true, |b| m.score_pct > b.score_pct) {
                            best_match = Some(m);
                        }
                    }
                }
            }

            if let Some(m) = best_match {
                if m.score_pct >= min_pct {
                    let score_pct = m.score_pct as i64;
                    let status = if score_pct >= 100 {
                        SegmentStatus::Translated
                    } else {
                        SegmentStatus::Draft
                    };

                    let words = count_words(&seg.source_text);
                    db.update_segment_full(
                        &seg.id,
                        &m.target_text,
                        &status,
                        Some(score_pct),
                        Some(&m.tm_name),
                        &now,
                    ).map_err(|e| e.to_string())?;

                    pretranslated += 1;
                    words_filled += words;
                }
            }
        }
    }

    log::info!("Pre-translation finished: {}/{} segments filled", pretranslated, total_segments);
    Ok(PreTranslateResult {
        total_segments,
        pretranslated,
        words_filled,
    })
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}
