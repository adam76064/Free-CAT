// core/tm/fragment.rs
// Sub-segment and fragment TM recall engine.
// Generates source n-grams (3 to 6 words) and queries SQLite FTS5 for sub-segment phrase matches.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentMatch {
    pub source_fragment: String,
    pub target_fragment: String,
    pub source_sentence: String,
    pub target_sentence: String,
    pub tu_id: i64,
    pub tm_id: String,
    pub tm_name: String,
}

/// Extract candidate n-grams of word lengths between min_words and max_words.
pub fn extract_source_ngrams(text: &str, min_words: usize, max_words: usize) -> Vec<String> {
    let words: Vec<&str> = text
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
        .filter(|w| !w.is_empty())
        .collect();

    if words.len() < min_words {
        return Vec::new();
    }

    let mut ngrams = Vec::new();
    let max_len = max_words.min(words.len());

    // Generate n-grams from longest to shortest (prefer longer exact phrase matches)
    for len in (min_words..=max_len).rev() {
        for window in words.windows(len) {
            let phrase = window.join(" ");
            // Only consider phrases with substantive length
            if phrase.len() >= 8 && !ngrams.contains(&phrase) {
                ngrams.push(phrase);
            }
        }
    }

    // Limit to top 15 candidate phrases to keep search lightning-fast (< 5ms total)
    ngrams.truncate(15);
    ngrams
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ngrams() {
        let text = "The quick brown fox jumps over the lazy dog";
        let ngrams = extract_source_ngrams(text, 3, 5);
        assert!(!ngrams.is_empty());
        assert!(ngrams.contains(&"The quick brown fox jumps".to_string()) || ngrams.contains(&"The quick brown fox".to_string()));
    }
}
