// core/tm/fuzzy.rs
// Fuzzy matching score engine.
//
// Formula (from plan research):
//   score = 0.7 × normalized_levenshtein + 0.3 × dice_coefficient
//
// References:
//   - strsim crate: https://crates.io/crates/strsim
//   - "Fuzzy Match Repair" research — ACL Anthology 2024
//   - Industry standard: SDL Trados uses Levenshtein; Déjà Vu uses Dice blend

use strsim::normalized_damerau_levenshtein;

/// Raw fuzzy scores before blending.
#[derive(Debug, Clone)]
pub struct FuzzyScore {
    pub levenshtein: f64,  // normalized edit distance similarity (0.0–1.0)
    pub dice:        f64,  // word-level bigram Dice coefficient (0.0–1.0)
    pub combined:    f64,  // 0.7 × levenshtein + 0.3 × dice
}

/// Compute the blended fuzzy match score between `query` and `candidate`.
///
/// Both strings are normalized (trimmed, collapsed whitespace) before scoring.
/// Uses Damerau-Levenshtein (handles transpositions like "teh" → "the") instead
/// of plain Levenshtein for better human-edit simulation.
pub fn score_match(query: &str, candidate: &str) -> FuzzyScore {
    let q = normalize_text(query);
    let c = normalize_text(candidate);

    // Shortcut: identical strings
    if q == c {
        return FuzzyScore { levenshtein: 1.0, dice: 1.0, combined: 1.0 };
    }

    // Empty string edge case
    if q.is_empty() || c.is_empty() {
        return FuzzyScore { levenshtein: 0.0, dice: 0.0, combined: 0.0 };
    }

    let lev  = normalized_damerau_levenshtein(&q, &c);
    let dice = word_dice(&q, &c);
    let combined = 0.7 * lev + 0.3 * dice;

    FuzzyScore { levenshtein: lev, dice, combined }
}

/// Convert a score (0.0–1.0) to a display percentage (0–102).
/// 102% = context match (set externally); 101% = structure context match.
pub fn score_to_pct(score: f64) -> u8 {
    (score * 100.0).round().min(100.0) as u8
}

/// Word-level Dice coefficient: bigram overlap of word tokens.
/// Better than character Dice for sentences — word order and vocabulary matter more.
fn word_dice(a: &str, b: &str) -> f64 {
    let a_words: Vec<&str> = a.split_whitespace().collect();
    let b_words: Vec<&str> = b.split_whitespace().collect();

    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }

    // Build bigram sets from word tokens
    let a_bigrams: std::collections::HashSet<(&str, &str)> = a_words.windows(2)
        .map(|w| (w[0], w[1]))
        .collect();
    let b_bigrams: std::collections::HashSet<(&str, &str)> = b_words.windows(2)
        .map(|w| (w[0], w[1]))
        .collect();

    // For single-word inputs, fall back to word-level intersection
    if a_words.len() == 1 || b_words.len() == 1 {
        let a_set: std::collections::HashSet<&str> = a_words.iter().copied().collect();
        let b_set: std::collections::HashSet<&str> = b_words.iter().copied().collect();
        let intersection = a_set.intersection(&b_set).count() as f64;
        return (2.0 * intersection) / (a_set.len() + b_set.len()) as f64;
    }

    let intersection = a_bigrams.intersection(&b_bigrams).count() as f64;
    (2.0 * intersection) / (a_bigrams.len() + b_bigrams.len()) as f64
}

/// Normalize text for comparison: trim, collapse whitespace, lowercase.
fn normalize_text(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_score_100() {
        let s = score_match("The quick brown fox", "The quick brown fox");
        assert_eq!(s.combined, 1.0);
    }

    #[test]
    fn empty_strings_score_zero() {
        let s = score_match("", "hello");
        assert_eq!(s.combined, 0.0);
    }

    #[test]
    fn high_similarity_above_threshold() {
        // One word changed
        let s = score_match(
            "The document must be translated by Friday.",
            "The document must be translated by Monday."
        );
        println!("One word diff: {:.2}%", s.combined * 100.0);
        assert!(s.combined >= 0.75, "Expected >= 75%, got {:.2}%", s.combined * 100.0);
    }

    #[test]
    fn low_similarity_below_threshold() {
        let s = score_match("Hello world", "Completely different sentence here.");
        println!("Low sim: {:.2}%", s.combined * 100.0);
        assert!(s.combined < 0.5);
    }

    #[test]
    fn transposition_handled() {
        // Damerau-Levenshtein handles "teh" → "the"
        let s = score_match("Please submit teh report", "Please submit the report");
        println!("Transposition: {:.2}%", s.combined * 100.0);
        assert!(s.levenshtein >= 0.95, "Damerau-Levenshtein should catch transposition: {:.2}%", s.levenshtein * 100.0);
        assert!(s.combined >= 0.75, "Combined score should be >= 75%: {:.2}%", s.combined * 100.0);
    }
}
