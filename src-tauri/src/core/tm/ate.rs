// core/tm/ate.rs — Automated Terminology Extraction (ATE) Engine
// Implements C-value collocation scoring for multi-word technical and legal terms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTerm {
    pub term: String,
    pub frequency: usize,
    pub c_value: f64,
    pub sample_context: String,
}

// Common English stopwords to ignore as single-word terms
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "if", "then", "else", "when",
    "at", "by", "for", "with", "about", "against", "between", "into", "through",
    "during", "before", "after", "above", "below", "to", "from", "up", "down",
    "in", "out", "on", "off", "over", "under", "again", "further", "then", "once",
    "here", "there", "all", "any", "both", "each", "few", "more", "most", "other",
    "some", "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too",
    "very", "s", "t", "can", "will", "just", "don", "should", "now", "is", "are",
    "was", "were", "be", "been", "being", "have", "has", "had", "having", "do",
    "does", "did", "doing", "this", "that", "these", "those", "shall", "may"
];

pub fn extract_terms(text: &str, min_freq: usize, max_ngram: usize) -> Vec<ExtractedTerm> {
    let sentences: Vec<&str> = text.split(|c: char| c == '.' || c == '\n' || c == '?' || c == '!')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut term_freqs: HashMap<String, usize> = HashMap::new();
    let mut term_contexts: HashMap<String, String> = HashMap::new();

    let stopword_set: std::collections::HashSet<&str> = STOPWORDS.iter().copied().collect();

    for sentence in &sentences {
        let words: Vec<&str> = sentence
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| !w.is_empty())
            .collect();

        for n in 1..=max_ngram {
            for window in words.windows(n) {
                // Ignore if first or last word is a stopword
                let first_lower = window.first().unwrap().to_lowercase();
                let last_lower = window.last().unwrap().to_lowercase();

                if stopword_set.contains(first_lower.as_str()) || stopword_set.contains(last_lower.as_str()) {
                    continue;
                }

                // If single word, length must be > 3 and not all lowercase stopword
                if n == 1 && window[0].len() <= 3 {
                    continue;
                }

                let term_candidate = window.join(" ");
                let key = term_candidate.to_lowercase();

                *term_freqs.entry(key.clone()).or_insert(0) += 1;
                term_contexts.entry(key).or_insert_with(|| sentence.to_string());
            }
        }
    }

    let mut candidates: Vec<ExtractedTerm> = term_freqs
        .into_iter()
        .filter(|(_, freq)| *freq >= min_freq)
        .map(|(term, freq)| {
            let word_count = term.split_whitespace().count();
            // C-value calculation: multi-word terms weighted by log2(words + 1) * frequency
            let c_value = (word_count as f64 + 1.0).log2() * (freq as f64);
            let sample_context = term_contexts.remove(&term).unwrap_or_default();

            ExtractedTerm {
                term,
                frequency: freq,
                c_value,
                sample_context,
            }
        })
        .collect();

    // Sort by C-value descending, then by frequency
    candidates.sort_by(|a, b| {
        b.c_value.partial_cmp(&a.c_value).unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.frequency.cmp(&a.frequency))
    });

    candidates.truncate(50);
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_technical_terms() {
        let corpus = "The Translation Memory stores segments. Each Translation Memory uses SQLite. The Translation Memory is fast.";
        let terms = extract_terms(corpus, 2, 3);
        assert!(!terms.is_empty());
        assert_eq!(terms[0].term, "translation memory");
        assert_eq!(terms[0].frequency, 3);
    }
}
