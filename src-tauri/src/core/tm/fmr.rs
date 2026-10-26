// core/tm/fmr.rs — Intelligent Fuzzy Match Repair (FMR) Engine
// Rule-based deterministic entity patching for numbers, dates, currency, and termbase pairs.
// Academic reference: Nieminen, Tiedemann & Virpioja (NoDaLiDa 2025).

use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmrChange {
    pub original_token: String,
    pub replacement_token: String,
    pub change_type: String, // "number", "term", "punctuation"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FmrResult {
    pub original_target: String,
    pub repaired_target: String,
    pub changes: Vec<FmrChange>,
    pub confidence: u8,
}

/// Attempt deterministic rule-based repair on a fuzzy match.
/// Checks for numeric discrepancies, termbase substitutions, and entity replacements.
pub fn repair_fuzzy_match(
    query_source: &str,
    tm_source: &str,
    tm_target: &str,
    known_terms: &[(String, String)], // (source_term, target_term)
) -> Option<FmrResult> {
    let mut repaired = tm_target.to_string();
    let mut changes = Vec::new();

    // 1. Numeric Entity Repair
    static NUM_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let num_re = NUM_RE.get_or_init(|| Regex::new(r"\b\d+([.,]\d+)?\b").unwrap());
    let query_nums: Vec<&str> = num_re.find_iter(query_source).map(|m| m.as_str()).collect();
    let tm_source_nums: Vec<&str> = num_re.find_iter(tm_source).map(|m| m.as_str()).collect();

    let to_arabic_indic = |s: &str| -> String {
        s.chars().map(|c| match c {
            '0' => '٠', '1' => '١', '2' => '٢', '3' => '٣', '4' => '٤',
            '5' => '٥', '6' => '٦', '7' => '٧', '8' => '٨', '9' => '٩',
            other => other,
        }).collect()
    };

    // If both source sentences contain the same count of numbers but with differing values:
    if !query_nums.is_empty() && query_nums.len() == tm_source_nums.len() {
        for (q_num, tm_num) in query_nums.iter().zip(tm_source_nums.iter()) {
            if q_num != tm_num {
                if repaired.contains(tm_num) {
                    // Replace the old TM number in the target with the query number
                    repaired = repaired.replacen(tm_num, q_num, 1);
                    changes.push(FmrChange {
                        original_token: tm_num.to_string(),
                        replacement_token: q_num.to_string(),
                        change_type: "number".into(),
                    });
                } else {
                    let tm_num_ar = to_arabic_indic(tm_num);
                    let q_num_ar = to_arabic_indic(q_num);
                    if repaired.contains(&tm_num_ar) {
                        repaired = repaired.replacen(&tm_num_ar, &q_num_ar, 1);
                        changes.push(FmrChange {
                            original_token: tm_num_ar,
                            replacement_token: q_num_ar,
                            change_type: "number".into(),
                        });
                    }
                }
            }
        }
    }

    // 2. Termbase Substitution
    for (src_term, tgt_term) in known_terms {
        // If query source contains src_term, but tm_source does not, check if tm_target can be patched
        if query_source.to_lowercase().contains(&src_term.to_lowercase())
            && !tm_source.to_lowercase().contains(&src_term.to_lowercase())
        {
            // Find what term in tm_source was replaced
            for (alt_src, alt_tgt) in known_terms {
                if alt_src != src_term && tm_source.to_lowercase().contains(&alt_src.to_lowercase()) {
                    if repaired.contains(alt_tgt) {
                        repaired = repaired.replacen(alt_tgt, tgt_term, 1);
                        changes.push(FmrChange {
                            original_token: alt_tgt.to_string(),
                            replacement_token: tgt_term.to_string(),
                            change_type: "term".into(),
                        });
                        break;
                    }
                }
            }
        }
    }

    if !changes.is_empty() && repaired != tm_target {
        Some(FmrResult {
            original_target: tm_target.to_string(),
            repaired_target: repaired,
            changes,
            confidence: 95,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_repair() {
        let query_src = "The Tenant shall pay $5,000 on the 1st of each month.";
        let tm_src = "The Tenant shall pay $3,000 on the 1st of each month.";
        let tm_tgt = "يدفع المستأجر 3,000 دولار في اليوم الأول من كل شهر.";

        let res = repair_fuzzy_match(query_src, tm_src, tm_tgt, &[]);
        assert!(res.is_some());
        let fmr = res.unwrap();
        assert_eq!(fmr.repaired_target, "يدفع المستأجر 5,000 دولار في اليوم الأول من كل شهر.");
        assert_eq!(fmr.changes.len(), 1);
        assert_eq!(fmr.changes[0].original_token, "3,000");
        assert_eq!(fmr.changes[0].replacement_token, "5,000");
    }

    #[test]
    fn test_term_substitution_repair() {
        let query_src = "The contract shall terminate automatically.";
        let tm_src = "The agreement shall terminate automatically.";
        let tm_tgt = "ينتهي الاتفاق تلقائياً.";
        let terms = vec![
            ("contract".into(), "العقد".into()),
            ("agreement".into(), "الاتفاق".into()),
        ];

        let res = repair_fuzzy_match(query_src, tm_src, tm_tgt, &terms);
        assert!(res.is_some());
        let fmr = res.unwrap();
        assert_eq!(fmr.repaired_target, "ينتهي العقد تلقائياً.");
        assert_eq!(fmr.changes[0].change_type, "term");
    }

    #[test]
    fn test_trial_segment_5_to_6_repair() {
        let query_src = "This is sentence 6 in this document.";
        let tm_src = "This is sentence 5 in this document.";
        let tm_tgt = "هذه جملة رقم 5 في الوثيقة.";

        let res = repair_fuzzy_match(query_src, tm_src, tm_tgt, &[]);
        assert!(res.is_some());
        let fmr = res.unwrap();
        assert_eq!(fmr.repaired_target, "هذه جملة رقم 6 في الوثيقة.");
        assert_eq!(fmr.changes[0].original_token, "5");
        assert_eq!(fmr.changes[0].replacement_token, "6");
    }

    #[test]
    fn test_arabic_indic_numeral_repair() {
        let query_src = "Chapter 8 summary.";
        let tm_src = "Chapter 7 summary.";
        let tm_tgt = "ملخص الفصل ٧.";

        let res = repair_fuzzy_match(query_src, tm_src, tm_tgt, &[]);
        assert!(res.is_some());
        let fmr = res.unwrap();
        assert_eq!(fmr.repaired_target, "ملخص الفصل ٨.");
        assert_eq!(fmr.changes[0].original_token, "٧");
        assert_eq!(fmr.changes[0].replacement_token, "٨");
    }
}
