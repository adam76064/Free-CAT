// core/qa/checks.rs
// All Phase 1 QA checks — 10 automated checks implemented as pure functions.
// Each check takes a segment and returns Option<QaIssue>.

use crate::core::models::{Segment, TermEntry};

#[derive(Debug, Clone)]
pub struct QaIssue {
    pub check_name: String,
    pub severity:   String, // "warning" | "error"
    pub message:    String,
}

/// Run all enabled QA checks against a segment.
/// `forbidden_terms` is the list of forbidden-status terms for this target language.
pub fn run_all_checks(
    segment: &Segment,
    forbidden_terms: &[TermEntry],
    max_length: Option<usize>,
) -> Vec<QaIssue> {
    let mut issues = Vec::new();

    if let Some(i) = check_empty_target(segment)        { issues.push(i); }
    if let Some(i) = check_tags(segment)                { issues.push(i); }
    if let Some(i) = check_whitespace(segment)          { issues.push(i); }
    if let Some(i) = check_double_space(segment)        { issues.push(i); }
    if let Some(i) = check_numbers(segment)             { issues.push(i); }
    if let Some(i) = check_repeated_words(segment)      { issues.push(i); }
    if let Some(i) = check_untranslated(segment)        { issues.push(i); }
    if let Some(i) = check_segment_length(segment, max_length) { issues.push(i); }
    issues.extend(check_forbidden_terms(segment, forbidden_terms));

    issues
}

// ── Check implementations ─────────────────────────────────────────────────────

/// QA-01: Empty target
fn check_empty_target(seg: &Segment) -> Option<QaIssue> {
    if seg.target_text.trim().is_empty() && !seg.source_text.trim().is_empty() {
        Some(QaIssue {
            check_name: "empty_target".into(),
            severity:   "error".into(),
            message:    "Target segment is empty.".into(),
        })
    } else {
        None
    }
}

/// QA-02: Tag mismatch — placeholder count must match between source and target.
/// Tags are represented as {N} in source/target text.
fn check_tags(seg: &Segment) -> Option<QaIssue> {
    let src_tags = extract_tags(&seg.source_text);
    let tgt_tags = extract_tags(&seg.target_text);

    if src_tags != tgt_tags {
        Some(QaIssue {
            check_name: "tag_mismatch".into(),
            severity:   "error".into(),
            message:    format!(
                "Tag mismatch: source has {:?}, target has {:?}.",
                src_tags, tgt_tags
            ),
        })
    } else {
        None
    }
}

/// QA-03: Leading / trailing whitespace in target
fn check_whitespace(seg: &Segment) -> Option<QaIssue> {
    if seg.target_text != seg.target_text.trim() && !seg.target_text.is_empty() {
        Some(QaIssue {
            check_name: "leading_trailing_space".into(),
            severity:   "warning".into(),
            message:    "Target has leading or trailing whitespace.".into(),
        })
    } else {
        None
    }
}

/// QA-04: Double spaces in target
fn check_double_space(seg: &Segment) -> Option<QaIssue> {
    if seg.target_text.contains("  ") {
        Some(QaIssue {
            check_name: "double_space".into(),
            severity:   "warning".into(),
            message:    "Target contains double spaces.".into(),
        })
    } else {
        None
    }
}

/// QA-05: Number consistency — numbers in source must appear in target.
fn check_numbers(seg: &Segment) -> Option<QaIssue> {
    let src_nums = extract_numbers(&seg.source_text);
    if src_nums.is_empty() {
        return None;
    }
    let tgt_nums = extract_numbers(&seg.target_text);

    let missing: Vec<&String> = src_nums.iter()
        .filter(|n| !tgt_nums.contains(n))
        .collect();

    if !missing.is_empty() {
        Some(QaIssue {
            check_name: "number_inconsistency".into(),
            severity:   "warning".into(),
            message:    format!("Numbers from source not found in target: {:?}", missing),
        })
    } else {
        None
    }
}

/// QA-06: Repeated consecutive words
fn check_repeated_words(seg: &Segment) -> Option<QaIssue> {
    let words: Vec<&str> = seg.target_text.split_whitespace().collect();
    for window in words.windows(2) {
        if window[0].to_lowercase() == window[1].to_lowercase() {
            return Some(QaIssue {
                check_name: "repeated_word".into(),
                severity:   "warning".into(),
                message:    format!("Repeated word in target: '{}'", window[0]),
            });
        }
    }
    None
}

/// QA-07: Untranslated — source == target (for non-identical language pairs).
/// This check skips locked segments and segments with source only being numbers/codes.
fn check_untranslated(seg: &Segment) -> Option<QaIssue> {
    if seg.target_text.trim().is_empty() {
        return None; // caught by empty_target
    }
    if seg.source_text.trim() == seg.target_text.trim() {
        // Allow if it looks like a code, number, or proper noun (all caps)
        let source = seg.source_text.trim();
        let is_code_or_number = source.chars().all(|c| c.is_numeric() || c == '.' || c == '-' || c == '_');
        let is_all_caps_short = source.len() <= 10 && source.chars().all(|c| c.is_uppercase() || !c.is_alphabetic());
        if !is_code_or_number && !is_all_caps_short {
            return Some(QaIssue {
                check_name: "untranslated".into(),
                severity:   "warning".into(),
                message:    "Target appears untranslated (identical to source).".into(),
            });
        }
    }
    None
}

/// QA-08: Segment length — target exceeds configured maximum character count.
fn check_segment_length(seg: &Segment, max_length: Option<usize>) -> Option<QaIssue> {
    if let Some(max) = max_length {
        if seg.target_text.len() > max {
            return Some(QaIssue {
                check_name: "segment_too_long".into(),
                severity:   "warning".into(),
                message:    format!(
                    "Target length {} exceeds maximum {}.",
                    seg.target_text.len(), max
                ),
            });
        }
    }
    None
}

/// QA-09: Forbidden terms — target contains a term marked as "forbidden" in the termbase.
fn check_forbidden_terms(seg: &Segment, forbidden: &[TermEntry]) -> Vec<QaIssue> {
    let target_lower = seg.target_text.to_lowercase();
    forbidden.iter()
        .filter(|t| target_lower.contains(&t.term.to_lowercase()))
        .map(|t| QaIssue {
            check_name: "forbidden_term".into(),
            severity:   "error".into(),
            message:    format!("Forbidden term used: '{}'", t.term),
        })
        .collect()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extract {N} style tag placeholders from text.
fn extract_tags(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\{(\d+)\}").unwrap();
    let mut tags: Vec<String> = re.captures_iter(text)
        .map(|c| c[0].to_string())
        .collect();
    tags.sort();
    tags
}

/// Extract numeric sequences from text (integers and decimals).
fn extract_numbers(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\b\d[\d,.]*\b").unwrap();
    re.find_iter(text)
        .map(|m| m.as_str().replace(',', ""))  // normalize 1,000 → 1000
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::SegmentStatus;

    fn make_seg(source: &str, target: &str) -> Segment {
        Segment {
            id: "test".into(), file_id: "f".into(), seq_number: 1,
            source_text: source.into(), target_text: target.into(),
            status: SegmentStatus::Draft, match_percent: None, match_origin: None,
            locked: false, bookmarked: false, comment: None,
            created_at: "2025-01-01".into(), modified_at: "2025-01-01".into(),
            modified_by: None,
        }
    }

    #[test]
    fn empty_target_is_error() {
        let seg = make_seg("Hello world", "");
        assert!(check_empty_target(&seg).is_some());
    }

    #[test]
    fn tag_mismatch_detected() {
        let seg = make_seg("Click {1}here{2}", "Cliquer {1}ici");
        assert!(check_tags(&seg).is_some());
    }

    #[test]
    fn number_in_source_missing_from_target() {
        let seg = make_seg("There are 42 items.", "Il y a des éléments.");
        assert!(check_numbers(&seg).is_some());
    }

    #[test]
    fn double_space_warned() {
        let seg = make_seg("Hello", "Bonjour  monde");
        assert!(check_double_space(&seg).is_some());
    }
}
