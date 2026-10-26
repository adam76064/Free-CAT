// core/aligner/dtw.rs — Dynamic Programming / DTW Bilingual Sentence Aligner
// Based on Gale-Church and Vecalign length-ratio dynamic programming with anchor detection.

use serde::{Deserialize, Serialize};
use regex::Regex;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignedPair {
    pub id: String,
    pub source_text: String,
    pub target_text: String,
    pub score: u8,
    pub alignment_type: String, // "1:1", "2:1", "1:2", "1:0", "0:1"
    pub confirmed: bool,
}

/// Extract numbers and numeric entities from text to serve as strong anchors
fn extract_anchors(text: &str) -> HashSet<String> {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\b\d+([.,]\d+)?\b").unwrap());
    re.find_iter(text)
        .map(|m| m.as_str().replace(',', "."))
        .collect()
}

/// Calculate Gale-Church distance between source sentence length and target sentence length.
/// c = source character count, l = target character count, r = mean length ratio.
fn distance_cost(src: &str, tgt: &str, mean_ratio: f64) -> f64 {
    let c = src.chars().count() as f64;
    let l = tgt.chars().count() as f64;

    if c == 0.0 && l == 0.0 {
        return 0.0;
    }
    if c == 0.0 || l == 0.0 {
        return 450.0; // Penalty for deletion/insertion
    }

    let variance = 6.8; // Empirical character variance
    let expected_l = c * mean_ratio;
    let delta = (l - expected_l).abs() / ((c * variance).sqrt().max(1.0));

    // Anchor similarity bonus: identical numbers and codes
    let src_anchors = extract_anchors(src);
    let tgt_anchors = extract_anchors(tgt);
    let shared_anchors = src_anchors.intersection(&tgt_anchors).count();

    let mut cost = delta * 100.0;
    if shared_anchors > 0 {
        cost = (cost - (shared_anchors as f64 * 80.0)).max(0.0);
    }

    cost
}

/// Align a vector of source sentences with target sentences using Dynamic Programming.
pub fn align_sentences(
    src_sentences: &[String],
    tgt_sentences: &[String],
) -> Vec<AlignedPair> {
    let n = src_sentences.len();
    let m = tgt_sentences.len();

    if n == 0 && m == 0 {
        return vec![];
    }
    if n == 0 {
        return tgt_sentences.iter().map(|tgt| AlignedPair {
            id: Uuid::new_v4().to_string(),
            source_text: String::new(),
            target_text: tgt.clone(),
            score: 0,
            alignment_type: "0:1".into(),
            confirmed: false,
        }).collect();
    }
    if m == 0 {
        return src_sentences.iter().map(|src| AlignedPair {
            id: Uuid::new_v4().to_string(),
            source_text: src.clone(),
            target_text: String::new(),
            score: 0,
            alignment_type: "1:0".into(),
            confirmed: false,
        }).collect();
    }

    // Calculate empirical target-to-source length ratio
    let total_src_len: usize = src_sentences.iter().map(|s| s.chars().count()).sum();
    let total_tgt_len: usize = tgt_sentences.iter().map(|t| t.chars().count()).sum();
    let mean_ratio = if total_src_len > 0 {
        (total_tgt_len as f64) / (total_src_len as f64)
    } else {
        1.0
    };

    // DP cost table and backpointer table
    // Transitions:
    // 0: (i-1, j-1) -> 1:1
    // 1: (i-2, j-1) -> 2:1
    // 2: (i-1, j-2) -> 1:2
    // 3: (i-1, j)   -> 1:0 (source deletion)
    // 4: (i, j-1)   -> 0:1 (target insertion)

    let mut dp = vec![vec![f64::INFINITY; m + 1]; n + 1];
    let mut back = vec![vec![0u8; m + 1]; n + 1];

    dp[0][0] = 0.0;

    for i in 0..=n {
        for j in 0..=m {
            if i == 0 && j == 0 {
                continue;
            }

            let mut best_cost = f64::INFINITY;
            let mut best_move = 0u8;

            // 1:1 match
            if i >= 1 && j >= 1 {
                let cost = dp[i - 1][j - 1] + distance_cost(&src_sentences[i - 1], &tgt_sentences[j - 1], mean_ratio);
                if cost < best_cost {
                    best_cost = cost;
                    best_move = 0;
                }
            }

            // 2:1 contraction (two source sentences to one target)
            if i >= 2 && j >= 1 {
                let combined_src = format!("{} {}", src_sentences[i - 2], src_sentences[i - 1]);
                let cost = dp[i - 2][j - 1] + distance_cost(&combined_src, &tgt_sentences[j - 1], mean_ratio) + 20.0;
                if cost < best_cost {
                    best_cost = cost;
                    best_move = 1;
                }
            }

            // 1:2 expansion (one source sentence to two target)
            if i >= 1 && j >= 2 {
                let combined_tgt = format!("{} {}", tgt_sentences[j - 2], tgt_sentences[j - 1]);
                let cost = dp[i - 1][j - 2] + distance_cost(&src_sentences[i - 1], &combined_tgt, mean_ratio) + 20.0;
                if cost < best_cost {
                    best_cost = cost;
                    best_move = 2;
                }
            }

            // 1:0 source omission
            if i >= 1 {
                let cost = dp[i - 1][j] + 350.0;
                if cost < best_cost {
                    best_cost = cost;
                    best_move = 3;
                }
            }

            // 0:1 target insertion
            if j >= 1 {
                let cost = dp[i][j - 1] + 350.0;
                if cost < best_cost {
                    best_cost = cost;
                    best_move = 4;
                }
            }

            dp[i][j] = best_cost;
            back[i][j] = best_move;
        }
    }

    // Backtrack to assemble aligned pairs
    let mut aligned = Vec::new();
    let mut curr_i = n;
    let mut curr_j = m;

    while curr_i > 0 || curr_j > 0 {
        let mv = back[curr_i][curr_j];
        match mv {
            0 => {
                // 1:1
                let s = &src_sentences[curr_i - 1];
                let t = &tgt_sentences[curr_j - 1];
                let cost = distance_cost(s, t, mean_ratio);
                let score = (100.0 - (cost / 4.0)).clamp(30.0, 100.0) as u8;
                aligned.push(AlignedPair {
                    id: Uuid::new_v4().to_string(),
                    source_text: s.clone(),
                    target_text: t.clone(),
                    score,
                    alignment_type: "1:1".into(),
                    confirmed: true,
                });
                curr_i -= 1;
                curr_j -= 1;
            }
            1 => {
                // 2:1
                let s = format!("{} {}", src_sentences[curr_i - 2], src_sentences[curr_i - 1]);
                let t = &tgt_sentences[curr_j - 1];
                let cost = distance_cost(&s, t, mean_ratio);
                let score = (95.0 - (cost / 4.0)).clamp(30.0, 95.0) as u8;
                aligned.push(AlignedPair {
                    id: Uuid::new_v4().to_string(),
                    source_text: s,
                    target_text: t.clone(),
                    score,
                    alignment_type: "2:1".into(),
                    confirmed: true,
                });
                curr_i -= 2;
                curr_j -= 1;
            }
            2 => {
                // 1:2
                let s = &src_sentences[curr_i - 1];
                let t = format!("{} {}", tgt_sentences[curr_j - 2], tgt_sentences[curr_j - 1]);
                let cost = distance_cost(s, &t, mean_ratio);
                let score = (95.0 - (cost / 4.0)).clamp(30.0, 95.0) as u8;
                aligned.push(AlignedPair {
                    id: Uuid::new_v4().to_string(),
                    source_text: s.clone(),
                    target_text: t,
                    score,
                    alignment_type: "1:2".into(),
                    confirmed: true,
                });
                curr_i -= 1;
                curr_j -= 2;
            }
            3 => {
                // 1:0
                aligned.push(AlignedPair {
                    id: Uuid::new_v4().to_string(),
                    source_text: src_sentences[curr_i - 1].clone(),
                    target_text: String::new(),
                    score: 0,
                    alignment_type: "1:0".into(),
                    confirmed: false,
                });
                curr_i -= 1;
            }
            4 => {
                // 0:1
                aligned.push(AlignedPair {
                    id: Uuid::new_v4().to_string(),
                    source_text: String::new(),
                    target_text: tgt_sentences[curr_j - 1].clone(),
                    score: 0,
                    alignment_type: "0:1".into(),
                    confirmed: false,
                });
                curr_j -= 1;
            }
            _ => break,
        }
    }

    aligned.reverse();
    aligned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perfect_1to1_alignment() {
        let src = vec![
            "Welcome to Free-CAT.".into(),
            "This software is 100% open-source.".into(),
            "Translate faster with AI.".into(),
        ];
        let tgt = vec![
            "مرحبًا بك في Free-CAT.".into(),
            "هذا البرنامج مفتوح المصدر بنسبة 100%.".into(),
            "ترجم بشكل أسرع مع الذكاء الاصطناعي.".into(),
        ];

        let pairs = align_sentences(&src, &tgt);
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0].alignment_type, "1:1");
        assert_eq!(pairs[1].alignment_type, "1:1");
        assert_eq!(pairs[2].alignment_type, "1:1");
        assert!(pairs[0].score >= 80);
    }

    #[test]
    fn test_numeric_anchor_alignment() {
        let src = vec![
            "Clause 10.5 applies here.".into(),
            "Clause 10.6 governs termination.".into(),
        ];
        let tgt = vec![
            "البند 10.5 ينطبق هنا.".into(),
            "البند 10.6 يحكم الإنهاء.".into(),
        ];

        let pairs = align_sentences(&src, &tgt);
        assert_eq!(pairs.len(), 2);
        assert!(pairs[0].source_text.contains("10.5"));
        assert!(pairs[0].target_text.contains("10.5"));
        assert!(pairs[1].source_text.contains("10.6"));
        assert!(pairs[1].target_text.contains("10.6"));
    }
}
