// core/tm/engine.rs
// High-level TM engine: queries multiple TMs, merges and ranks results,
// applies context match detection (101%/102%).

use anyhow::Result;
use crate::core::models::{TmMatchResult, TranslationUnit};
use crate::core::db::tm_db::TmDb;
use crate::core::tm::context::detect_context_match;

/// Query multiple TMs and return merged, ranked results for one source segment.
pub fn query_all_tms(
    tms: &[(&TmDb, f64)],  // (tm_db, penalty) pairs
    source_text: &str,
    src_lang: &str,
    tgt_lang: &str,
    threshold: f64,
    prev_segment: Option<&str>,    // for 101% context check
    next_segment: Option<&str>,    // for 101% context check
) -> Result<Vec<TmMatchResult>> {
    let mut all_results: Vec<TmMatchResult> = Vec::new();

    for (tm, penalty) in tms {
        let mut matches = tm.query(source_text, src_lang, tgt_lang, threshold, 5, *penalty)?;

        // Apply context match detection (101%/102%) to 100% exact matches
        for m in &mut matches {
            if m.score_pct == 100 {
                let ctx = detect_context_match(
                    &m.source_text,
                    source_text,
                    prev_segment,
                    next_segment,
                );
                if ctx > 100 {
                    m.score_pct = ctx;
                    m.is_context = true;
                }
            }
        }

        all_results.extend(matches);
    }

    // Sort: context matches (102/101) first, then exact (100), then fuzzy desc
    all_results.sort_by(|a, b| b.score_pct.cmp(&a.score_pct));

    // Deduplicate: keep only the highest-scoring match per unique target text
    let mut seen_targets = std::collections::HashSet::new();
    let deduped: Vec<TmMatchResult> = all_results
        .into_iter()
        .filter(|m| seen_targets.insert(m.target_text.clone()))
        .take(5)
        .collect();

    Ok(deduped)
}

/// Add a confirmed segment to all write-enabled TMs.
pub fn write_back(
    tms: &[&TmDb],
    source_text: &str,
    target_text: &str,
    src_lang: &str,
    tgt_lang: &str,
    source_file: Option<&str>,
) -> Result<()> {
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let tu = TranslationUnit {
        id:          None,
        source_lang: src_lang.to_string(),
        target_lang: tgt_lang.to_string(),
        source_text: source_text.to_string(),
        target_text: target_text.to_string(),
        created_at:  now.clone(),
        modified_at: now,
        created_by:  None,
        client:      None,
        domain:      None,
        source_file: source_file.map(|s| s.to_string()),
        usage_count: None,
        metadata:    None,
    };

    for tm in tms {
        tm.upsert(&tu)?;
    }
    Ok(())
}
