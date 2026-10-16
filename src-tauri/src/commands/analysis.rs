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

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AnalysisBand {
    pub id: String,
    pub name: String,
    pub match_range: String,
    pub segments: usize,
    pub words: usize,
    pub default_rate_pct: f64,
    pub weighted_words: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DetailedAnalysisReport {
    pub project_id: String,
    pub total_words: usize,
    pub total_segments: usize,
    pub bands: Vec<AnalysisBand>,
    pub total_weighted_words: f64,
}

/// Compute industry-standard multi-band analysis report for quoting and invoicing.
#[tauri::command]
pub async fn get_detailed_analysis(
    project_id: String,
    file_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<DetailedAnalysisReport, String> {
    let db = state.project_db.lock().unwrap();

    let mut repetitions_words = 0;
    let mut repetitions_segs = 0;
    let mut context_words = 0;
    let mut context_segs = 0;
    let mut exact_words = 0;
    let mut exact_segs = 0;
    let mut fuzzy_95_words = 0;
    let mut fuzzy_95_segs = 0;
    let mut fuzzy_85_words = 0;
    let mut fuzzy_85_segs = 0;
    let mut fuzzy_75_words = 0;
    let mut fuzzy_75_segs = 0;
    let mut fuzzy_50_words = 0;
    let mut fuzzy_50_segs = 0;
    let mut new_words = 0;
    let mut new_segs = 0;

    let mut total_words = 0;
    let mut total_segments = 0;
    let mut seen_source_hashes = std::collections::HashSet::new();

    for file_id in &file_ids {
        let segments = db.get_segments_for_file(file_id)
            .map_err(|e| e.to_string())?;

        for seg in &segments {
            let words = count_words(&seg.source_text);
            total_segments += 1;
            total_words += words;

            let norm_src = seg.source_text.trim().to_lowercase();
            if seen_source_hashes.contains(&norm_src) {
                repetitions_segs += 1;
                repetitions_words += words;
            } else {
                seen_source_hashes.insert(norm_src);
                match seg.match_percent {
                    Some(pct) if pct >= 101 => {
                        context_segs += 1;
                        context_words += words;
                    }
                    Some(pct) if pct >= 100 => {
                        exact_segs += 1;
                        exact_words += words;
                    }
                    Some(pct) if pct >= 95 => {
                        fuzzy_95_segs += 1;
                        fuzzy_95_words += words;
                    }
                    Some(pct) if pct >= 85 => {
                        fuzzy_85_segs += 1;
                        fuzzy_85_words += words;
                    }
                    Some(pct) if pct >= 75 => {
                        fuzzy_75_segs += 1;
                        fuzzy_75_words += words;
                    }
                    Some(pct) if pct >= 50 => {
                        fuzzy_50_segs += 1;
                        fuzzy_50_words += words;
                    }
                    _ => {
                        new_segs += 1;
                        new_words += words;
                    }
                }
            }
        }
    }

    let bands = vec![
        AnalysisBand {
            id: "context".into(),
            name: "Context Match".into(),
            match_range: "101%+".into(),
            segments: context_segs,
            words: context_words,
            default_rate_pct: 0.0,
            weighted_words: 0.0,
        },
        AnalysisBand {
            id: "repetitions".into(),
            name: "Repetitions".into(),
            match_range: "Repeated".into(),
            segments: repetitions_segs,
            words: repetitions_words,
            default_rate_pct: 0.30,
            weighted_words: (repetitions_words as f64) * 0.30,
        },
        AnalysisBand {
            id: "exact".into(),
            name: "Exact Match".into(),
            match_range: "100%".into(),
            segments: exact_segs,
            words: exact_words,
            default_rate_pct: 0.10,
            weighted_words: (exact_words as f64) * 0.10,
        },
        AnalysisBand {
            id: "fuzzy_95".into(),
            name: "Fuzzy 95% - 99%".into(),
            match_range: "95% - 99%".into(),
            segments: fuzzy_95_segs,
            words: fuzzy_95_words,
            default_rate_pct: 0.30,
            weighted_words: (fuzzy_95_words as f64) * 0.30,
        },
        AnalysisBand {
            id: "fuzzy_85".into(),
            name: "Fuzzy 85% - 94%".into(),
            match_range: "85% - 94%".into(),
            segments: fuzzy_85_segs,
            words: fuzzy_85_words,
            default_rate_pct: 0.50,
            weighted_words: (fuzzy_85_words as f64) * 0.50,
        },
        AnalysisBand {
            id: "fuzzy_75".into(),
            name: "Fuzzy 75% - 84%".into(),
            match_range: "75% - 84%".into(),
            segments: fuzzy_75_segs,
            words: fuzzy_75_words,
            default_rate_pct: 0.70,
            weighted_words: (fuzzy_75_words as f64) * 0.70,
        },
        AnalysisBand {
            id: "fuzzy_50".into(),
            name: "Fuzzy 50% - 74%".into(),
            match_range: "50% - 74%".into(),
            segments: fuzzy_50_segs,
            words: fuzzy_50_words,
            default_rate_pct: 1.00,
            weighted_words: (fuzzy_50_words as f64) * 1.00,
        },
        AnalysisBand {
            id: "new".into(),
            name: "New / No Match".into(),
            match_range: "0% - 49%".into(),
            segments: new_segs,
            words: new_words,
            default_rate_pct: 1.00,
            weighted_words: (new_words as f64) * 1.00,
        },
    ];

    let total_weighted_words: f64 = bands.iter().map(|b| b.weighted_words).sum();

    Ok(DetailedAnalysisReport {
        project_id,
        total_words,
        total_segments,
        bands,
        total_weighted_words,
    })
}

#[derive(serde::Deserialize)]
pub struct PseudoTranslateArgs {
    pub file_id: String,
    pub expansion_pct: Option<u32>,
    pub add_brackets: Option<bool>,
    pub use_diacritics: Option<bool>,
    pub overwrite_confirmed: Option<bool>,
}

#[derive(serde::Serialize)]
pub struct PseudoTranslateResult {
    pub segments_transformed: usize,
    pub total_words: usize,
}

/// Run pseudo-translation simulation on a file to test DTP expansion and diacritics.
#[tauri::command]
pub async fn pseudotranslate_file(
    args: PseudoTranslateArgs,
    state: State<'_, AppState>,
) -> Result<PseudoTranslateResult, String> {
    use crate::core::analysis::pseudotranslate::{pseudotranslate_text, PseudoTranslateOptions};

    let opts = PseudoTranslateOptions {
        expansion_pct: args.expansion_pct.unwrap_or(30),
        add_brackets: args.add_brackets.unwrap_or(true),
        use_diacritics: args.use_diacritics.unwrap_or(true),
    };
    let overwrite = args.overwrite_confirmed.unwrap_or(false);
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    let db = state.project_db.lock().unwrap();
    let segments = db.get_segments_for_file(&args.file_id)
        .map_err(|e| e.to_string())?;

    let mut transformed_count = 0;
    let mut total_words = 0;

    for seg in segments {
        if !overwrite && (seg.status == SegmentStatus::Translated || seg.status == SegmentStatus::Reviewed || seg.status == SegmentStatus::Final) {
            continue;
        }
        if seg.locked {
            continue;
        }

        let pseudo = pseudotranslate_text(&seg.source_text, &opts);
        total_words += count_words(&seg.source_text);
        transformed_count += 1;

        db.update_segment_full(
            &seg.id,
            &pseudo,
            &SegmentStatus::Draft,
            Some(100),
            Some("pseudo_translation"),
            &now,
        ).map_err(|e| e.to_string())?;
    }

    Ok(PseudoTranslateResult {
        segments_transformed: transformed_count,
        total_words,
    })
}

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}
